use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use lcc_auth::{Caller, Permission, SharedVerifier};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::domain::{
    ConnectionStatus, ContactTier, InteractionKind, RelationshipStage, RelationshipStrength,
};
use crate::error::Error;
use crate::service::{NewCompany, NewContact};
use crate::state::AppState;

/// Builds the CRM router and puts the authorization gate on it.
///
/// ## Authorization (the gap this closes)
///
/// Before this pass every handler called a local `require_member` that parsed
/// the bearer token and returned the `sub` claim — and nothing else. The role
/// in the token was never compared against anything, so `lcc_auth::rbac`'s
/// policy table (Owner/Admin/Assistant/Reviewer/Auditor) had no effect here: an
/// `Auditor`, whose documented duty is read-only, could `PATCH` a contact,
/// delete one, or log an interaction. The gateway verifies a token before
/// proxying, but a caller that reaches the service directly — cluster-internal
/// path, misconfigured ingress, an SSRF elsewhere — got the same unrestricted
/// access.
///
/// The gate is a `route_layer`, applied where the routes are registered rather
/// than inside each handler. Per-handler checks are what produced the original
/// gap: the next route added here would be unprotected unless somebody
/// remembered. With the layer, a new route is protected by default and cannot
/// be reached by a path that skips the handler body.
///
/// * reads (`GET`) require [`Permission::ViewOwnData`] — Owner, Admin,
///   Assistant, Reviewer and Auditor all hold it;
/// * writes require [`Permission::EditContent`] — Owner, Admin and Assistant.
///   Reviewer and Auditor get **403**; a missing or unusable credential gets
///   **401**.
///
/// `member_id` still comes only from the verified `sub` claim (see
/// `lcc_auth::identity`), never from a path or query parameter, so a caller
/// cannot reach another member's rows by supplying a different id. Every query
/// keeps its own `member_id` predicate regardless: a caller with write access
/// still cannot read or write a contact that is not theirs — that is a 404,
/// not a 403, so no existence oracle is created.
pub fn build_router(state: AppState, verifier: SharedVerifier) -> Router {
    let read = verifier.clone();
    let write = verifier;
    Router::new()
        .route(
            "/api/v1/contacts",
            get(list_contacts).post(create_contact),
        )
        .route(
            "/api/v1/contacts/:id",
            get(get_contact)
                .patch(update_contact)
                .delete(delete_contact),
        )
        .route(
            "/api/v1/contacts/:id/interactions",
            get(list_interactions).post(record_interaction),
        )
        .route("/api/v1/contacts/:id/company", axum::routing::patch(link_company))
        .route(
            "/api/v1/companies",
            get(list_companies).post(create_company),
        )
        .route("/api/v1/companies/:id", get(get_company))
        // Staleness is a contact query. It was mounted at
        // `/api/v1/companies/staleness`, which is both the wrong resource and
        // a POST-shaped path colliding with `/api/v1/companies/:id`.
        .route("/api/v1/contacts/stale", get(staleness))
        // Every route registered so far is gated on ViewOwnData...
        .route_layer(axum::middleware::from_fn(move |request, next| {
            let v = read.clone();
            async move {
                lcc_auth::guard::require_permission(v, Permission::ViewOwnData, request, next)
                    .await
            }
        }))
        // ...and the mutating ones additionally on EditContent.
        .route_layer(axum::middleware::from_fn(move |request, next| {
            let v = write.clone();
            async move {
                lcc_auth::guard::require_permission(v, Permission::EditContent, request, next)
                    .await
            }
        }))
        .with_state(state)
        // F-AUDIT-56: this merge was missing, so the service exposed no
        // health endpoint at all and its pod would have crash-looped on the
        // manifest's /healthz liveness probe.
        //
        // Merged AFTER the layers and `.with_state()`: the health router is
        // stateless, so merging it first would pin the router's state type to
        // `()`; merging it before the `route_layer`s would put the probe
        // behind authentication and the kubelet has no bearer token.
        .merge(crate::health::router())
}

#[derive(Deserialize)]
struct ListQuery {
    q: Option<String>,
    limit: Option<i64>,
}

async fn list_contacts(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let v = state
        .service()
        .list_contacts(caller.member_id, query.q.as_deref(), limit_of(query.limit))
        .await?;
    Ok(Json(json!({"contacts": v})))
}

async fn get_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
) -> Result<impl IntoResponse, Error> {
    Ok(Json(
        state.service().get_contact(caller.member_id, id).await?,
    ))
}

#[derive(Deserialize)]
struct CreateContactRequest {
    /// Canonical contract `ContactCreate.name`, sourced from
    /// `contacts.display_name`. See `Contact::display_name`.
    display_name: Option<String>,
    /// Accepted under the contract's name for the same field.
    name: Option<String>,
    title: Option<String>,
    headline: Option<String>,
    linkedin_id: Option<String>,
    linkedin_url: Option<String>,
    company_id: Option<Uuid>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    tier: Option<String>,
    first_contact_date: Option<NaiveDate>,
}

async fn create_contact(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<CreateContactRequest>,
) -> Result<impl IntoResponse, Error> {
    let display_name = req
        .display_name
        .or(req.name)
        .ok_or_else(|| Error::Validation("display_name required".into()))?;
    let c = state
        .service()
        .create_contact(
            caller.member_id,
            NewContact {
                display_name: &display_name,
                title: req.title.as_deref(),
                headline: req.headline.as_deref(),
                linkedin_id: req.linkedin_id.as_deref(),
                linkedin_url: req.linkedin_url.as_deref(),
                company_id: req.company_id,
                tags: req.tags.unwrap_or_default(),
                notes: req.notes.as_deref(),
                tier: Some(parse_tier(req.tier.as_deref())?),
                first_contact_date: req.first_contact_date,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(c)))
}

#[derive(Deserialize)]
struct UpdateContactRequest {
    /// Canonical contract `ContactUpdate.expected_version`. Required, not
    /// optional: design §52 makes the compare-and-swap the mechanism, and a
    /// silently-optional version would turn every concurrent write into a lost
    /// update.
    expected_version: Option<i32>,
    version: Option<i32>,
    display_name: Option<String>,
    name: Option<String>,
    title: Option<Option<String>>,
    headline: Option<Option<String>>,
    company: Option<Option<String>>,
    linkedin_id: Option<Option<String>>,
    linkedin_url: Option<Option<String>>,
    notes: Option<Option<String>>,
    tags: Option<Vec<String>>,
    company_id: Option<Option<Uuid>>,
    tier: Option<String>,
    is_mutual: Option<bool>,
    connection_status: Option<String>,
    relationship_stage: Option<String>,
    relationship_strength: Option<String>,
    first_contact_date: Option<Option<NaiveDate>>,
    follow_up_date: Option<Option<NaiveDate>>,
}

async fn update_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<impl IntoResponse, Error> {
    let expected_version = req
        .expected_version
        .or(req.version)
        .ok_or_else(|| Error::Validation("expected_version required".into()))?;
    let patch = crate::service::ContactPatch {
        display_name: req.display_name.or(req.name),
        title: req.title,
        headline: req.headline,
        company: req.company,
        linkedin_id: req.linkedin_id,
        linkedin_url: req.linkedin_url,
        notes: req.notes,
        tags: req.tags,
        company_id: req.company_id,
        tier: parse_tier(req.tier.as_deref()).ok(),
        is_mutual: req.is_mutual,
        connection_status: parse_connection_status(req.connection_status.as_deref()).ok(),
        relationship_stage: parse_stage(req.relationship_stage.as_deref()).ok(),
        relationship_strength: parse_strength(req.relationship_strength.as_deref()).ok(),
        first_contact_date: req.first_contact_date,
        follow_up_date: req.follow_up_date,
    };
    // A value that is present but unparseable must be rejected, not ignored.
    // `Option::ok()` above collapses "absent" and "invalid" into the same
    // `None`, which would answer 200 while silently discarding the caller's
    // write — so the parse failures are re-raised here.
    if req.tier.is_some() && patch.tier.is_none() {
        return Err(Error::Validation("unknown tier".into()));
    }
    if req.connection_status.is_some() && patch.connection_status.is_none() {
        return Err(Error::Validation("unknown connection_status".into()));
    }
    if req.relationship_stage.is_some() && patch.relationship_stage.is_none() {
        return Err(Error::Validation("unknown relationship_stage".into()));
    }
    if req.relationship_strength.is_some() && patch.relationship_strength.is_none() {
        return Err(Error::Validation("unknown relationship_strength".into()));
    }
    Ok(Json(
        state
            .service()
            .update_contact(caller.member_id, id, expected_version, patch)
            .await?,
    ))
}

async fn delete_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
) -> Result<impl IntoResponse, Error> {
    state.service().delete_contact(caller.member_id, id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_interactions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
) -> Result<impl IntoResponse, Error> {
    let v = state
        .service()
        .list_interactions(caller.member_id, id)
        .await?;
    Ok(Json(v))
}

#[derive(Deserialize)]
struct RecordInteractionRequest {
    kind: String,
    summary: String,
    /// `lcc.interactions.actor` (0017): `'member'` for a human entry,
    /// `'system:<service>'` for an auto-logged event.
    actor: Option<String>,
    /// Canonical contract `InteractionCreate.occurred_at`.
    occurred_at: Option<DateTime<Utc>>,
}

async fn record_interaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<RecordInteractionRequest>,
) -> Result<impl IntoResponse, Error> {
    if req.summary.trim().is_empty() {
        return Err(Error::Validation("summary required".into()));
    }
    let i = state
        .service()
        .record_interaction(
            caller.member_id,
            id,
            parse_kind(&req.kind)?,
            &req.summary,
            req.actor.as_deref().unwrap_or("member"),
            req.occurred_at.unwrap_or_else(Utc::now),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(i)))
}

#[derive(Deserialize)]
struct LinkCompanyRequest {
    company_id: Uuid,
}

async fn link_company(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<LinkCompanyRequest>,
) -> Result<impl IntoResponse, Error> {
    state
        .service()
        .link(caller.member_id, id, req.company_id)
        .await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

async fn list_companies(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, Error> {
    let v = state
        .service()
        .list_companies(caller.member_id, query.q.as_deref(), limit_of(query.limit))
        .await?;
    Ok(Json(json!({"companies": v})))
}

async fn get_company(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(caller): Extension<Caller>,
) -> Result<impl IntoResponse, Error> {
    Ok(Json(
        state.service().get_company(caller.member_id, id).await?,
    ))
}

#[derive(Deserialize)]
struct CreateCompanyRequest {
    name: String,
    domain: Option<String>,
    industry: Option<String>,
    size_band: Option<String>,
    funding_stage: Option<String>,
    hq_location: Option<String>,
    tech_stack: Option<Vec<String>>,
    /// JSONB in the schema (`companies.trigger_events`), so an object or array
    /// both round-trip. The pre-remediation handler bound these as
    /// `Vec<String>`, which cannot represent the column.
    trigger_events: Option<serde_json::Value>,
    public_signals: Option<serde_json::Value>,
    third_party_ttl_at: Option<DateTime<Utc>>,
}

async fn create_company(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
    Json(req): Json<CreateCompanyRequest>,
) -> Result<impl IntoResponse, Error> {
    let c = state
        .service()
        .create_company(
            caller.member_id,
            NewCompany {
                name: &req.name,
                domain: req.domain.as_deref(),
                industry: req.industry.as_deref(),
                size_band: req.size_band.as_deref(),
                funding_stage: req.funding_stage.as_deref(),
                hq_location: req.hq_location.as_deref(),
                tech_stack: req.tech_stack.unwrap_or_default(),
                trigger_events: req.trigger_events.unwrap_or_else(|| json!([])),
                public_signals: req.public_signals.unwrap_or_else(|| json!([])),
                third_party_ttl_at: req.third_party_ttl_at,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(c)))
}

async fn staleness(
    State(state): State<AppState>,
    Extension(caller): Extension<Caller>,
) -> Result<impl IntoResponse, Error> {
    Ok(Json(state.service().staleness(caller.member_id).await?))
}

fn limit_of(raw: Option<i64>) -> i64 {
    // A negative or absurd limit is a client error, not a licence to ask
    // Postgres for everything; clamp rather than trusting the query string.
    match raw {
        Some(n) if n > 0 => n.min(500),
        _ => 100,
    }
}

fn parse_kind(s: &str) -> Result<InteractionKind, Error> {
    // Exactly the values `interactions_kind_check` (0017) allows. The old
    // handler accepted `outbound_message`, `phone_call`, `reaction`,
    // `public_comment` and `other`, none of which the CHECK permits, so every
    // one of them was a 500 on write.
    s.parse::<InteractionKind>()
        .map_err(|()| Error::Validation(format!("unknown interaction kind {s:?}")))
}

fn parse_tier(s: Option<&str>) -> Result<ContactTier, Error> {
    match s {
        None => Ok(ContactTier::Standard),
        Some(v) => v
            .parse::<ContactTier>()
            .map_err(|()| Error::Validation(format!("unknown tier {v:?}"))),
    }
}

fn parse_connection_status(s: Option<&str>) -> Result<ConnectionStatus, Error> {
    match s {
        None => Err(Error::Validation("connection_status required".into())),
        Some(v) => v
            .parse::<ConnectionStatus>()
            .map_err(|()| Error::Validation(format!("unknown connection_status {v:?}"))),
    }
}

fn parse_stage(s: Option<&str>) -> Result<RelationshipStage, Error> {
    match s {
        None => Err(Error::Validation("relationship_stage required".into())),
        Some(v) => v
            .parse::<RelationshipStage>()
            .map_err(|()| Error::Validation(format!("unknown relationship_stage {v:?}"))),
    }
}

fn parse_strength(s: Option<&str>) -> Result<RelationshipStrength, Error> {
    match s {
        None => Err(Error::Validation("relationship_strength required".into())),
        Some(v) => v
            .parse::<RelationshipStrength>()
            .map_err(|()| Error::Validation(format!("unknown relationship_strength {v:?}"))),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::InteractionKind;

    #[test]
    fn every_kind_the_handler_accepts_satisfies_the_database_check() {
        // The handler and the CHECK constraint must not drift: each accepted
        // string has to be a member of the enum whose `as_str` is what the
        // INSERT binds.
        for s in [
            "manual_note",
            "inbound_message",
            "sent_message",
            "call",
            "meeting",
            "sequence_step_sent",
            "reply_received",
            "connection_accepted",
        ] {
            let k = parse_kind(s).unwrap();
            assert_eq!(k.as_str(), s);
        }
    }

    #[test]
    fn kinds_the_old_handler_accepted_are_now_rejected() {
        // Each of these passed `parse_kind` before and would have failed the
        // `interactions_kind_check` constraint at INSERT time — a 500.
        for s in [
            "outbound_message",
            "phone_call",
            "connection_request",
            "reaction",
            "public_comment",
            "other",
        ] {
            assert!(
                parse_kind(s).is_err(),
                "{s} is not in interactions_kind_check and must not parse"
            );
        }
    }

    #[test]
    fn enum_values_round_trip_through_parse() {
        assert_eq!(parse_tier(Some("VIP")).unwrap(), ContactTier::Vip);
        assert_eq!(parse_tier(Some("peer")).unwrap(), ContactTier::Peer);
        assert_eq!(parse_tier(None).unwrap(), ContactTier::Standard);
        assert!(parse_tier(Some("gold")).is_err());

        assert_eq!(
            parse_connection_status(Some("pending")).unwrap(),
            ConnectionStatus::Pending
        );
        assert!(parse_connection_status(Some("maybe")).is_err());

        assert_eq!(
            parse_stage(Some("conversation")).unwrap(),
            RelationshipStage::Conversation
        );
        assert!(parse_stage(Some("won")).is_err());

        assert_eq!(
            parse_strength(Some("strong_recent")).unwrap(),
            RelationshipStrength::StrongRecent
        );
        // A number is not a relationship strength: the pre-remediation domain
        // carried an i16, and the database never stored one.
        assert!(parse_strength(Some("4")).is_err());
    }

    #[test]
    fn limit_is_clamped_not_trusted() {
        assert_eq!(limit_of(None), 100);
        assert_eq!(limit_of(Some(0)), 100);
        assert_eq!(limit_of(Some(-5)), 100);
        assert_eq!(limit_of(Some(50)), 50);
        assert_eq!(limit_of(Some(10_000)), 500);
    }

    #[test]
    fn interaction_kind_as_str_matches_parse_kind() {
        assert_eq!(
            parse_kind(InteractionKind::ReplyReceived.as_str()).unwrap(),
            InteractionKind::ReplyReceived
        );
    }
}
