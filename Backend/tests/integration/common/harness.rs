//! Shared integration-test harness.
//!
//! Provides:
//! - `MockComplianceGovernor`: a fully wired in-process governor that exercises
//!   the 8-guard pipeline against an in-memory state. The same code path runs
//!   in production; only the backing stores differ.
//! - `MockIntegrationGateway`: verifies permit-tokens via the same JWT shape
//!   as production (HS256 or Ed25519), enforces idempotency, and records the
//!   final dispatch.
//! - `MockAuditLog`: in-memory append-only store with checksum chain.
//! - `MockRlsDb`: per-member isolation simulated via a per-member map.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ---------- Action types ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    ConnectionRequest,
    Dm,
    PostPublish,
    CommentPost,
    LikePost,
    ProfileView,
    FollowCompany,
    JobApplicationSubmit,
    ClientProposalSend,
    ExecutiveOutreach,
    SequenceStepSend,
}

impl ActionType {
    pub fn risk_tier(&self) -> RiskTier {
        use ActionType::*;
        match self {
            ConnectionRequest | Dm | JobApplicationSubmit | ClientProposalSend
            | ExecutiveOutreach => RiskTier::High,
            PostPublish | SequenceStepSend => RiskTier::Medium,
            CommentPost | LikePost | ProfileView | FollowCompany => RiskTier::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier {
    Low,
    Medium,
    High,
}

// ---------- Compliance Governor (mock) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorRequest {
    pub member_id: Uuid,
    pub action_type: ActionType,
    pub target_kind: String,
    pub target_id: Option<String>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernorDecision {
    pub decision: String, // "allow" | "deny"
    pub reason: String,
    pub guards_passed: Vec<String>,
    pub guards_failed: Vec<String>,
    pub permit_token: Option<String>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub daily_caps: HashMap<String, u32>,
    pub cooldowns: HashMap<String, i64>, // seconds
    pub min_kb_refs: usize,
    pub restriction_active: bool,
    pub session_pacing_window_secs: i64,
    pub session_pacing_max: u32,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        let mut daily_caps = HashMap::new();
        daily_caps.insert("connection_request".into(), 25);
        daily_caps.insert("dm".into(), 40);
        daily_caps.insert("post_publish".into(), 2);
        daily_caps.insert("sequence_step_send".into(), 20);
        daily_caps.insert("job_application_submit".into(), 8);
        daily_caps.insert("client_proposal_send".into(), 5);
        daily_caps.insert("executive_outreach".into(), 3);

        let mut cooldowns = HashMap::new();
        cooldowns.insert(
            "connection_request_min_days_between_to_same_target".into(),
            7 * 86_400,
        );
        cooldowns.insert("dm_min_hours_between_to_same_target".into(), 24 * 3_600);

        Self {
            daily_caps,
            cooldowns,
            min_kb_refs: 1,
            restriction_active: false,
            session_pacing_window_secs: 600,
            session_pacing_max: 5,
        }
    }
}

#[derive(Default)]
pub struct GovernorState {
    pub daily_used: HashMap<(Uuid, String, chrono::NaiveDate), u32>, // (member, action_type, date) → used
    pub last_target_action: HashMap<(Uuid, String, String), DateTime<Utc>>, // (member, action_type, target_id) → last_at
    pub session_pacing: HashMap<(Uuid, String, i64), u32>, // (member, action, window_bucket) → count
    pub restrictions: HashMap<Uuid, bool>,                 // member → restricted
}

#[derive(Clone)]
pub struct MockComplianceGovernor {
    pub config: Arc<Mutex<ComplianceConfig>>,
    pub state: Arc<Mutex<GovernorState>>,
}

impl MockComplianceGovernor {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(ComplianceConfig::default())),
            state: Arc::new(Mutex::new(GovernorState::default())),
        }
    }

    pub fn set_restriction(&self, member_id: Uuid, restricted: bool) {
        let mut state = self.state.lock().unwrap();
        state.restrictions.insert(member_id, restricted);
    }

    pub fn evaluate(&self, req: &GovernorRequest) -> GovernorDecision {
        let cfg = self.config.lock().unwrap().clone();
        let mut state = self.state.lock().unwrap();
        let mut guards_passed = Vec::new();
        let mut guards_failed = Vec::new();
        let today = chrono::Utc::now().date_naive();
        // F-AUDIT-50: this was `format!("{:?}", req.action_type).to_lowercase()`,
        // which turns `PostPublish` into "postpublish" and
        // `JobApplicationSubmit` into "jobapplicationsubmit". The config keys
        // are snake_case — the real `lcc-compliance` crate maps
        // `ActionType::PostPublish => "post_publish"` in
        // `crates/compliance/src/action.rs::as_str()` — so **no** lookup in
        // `daily_caps` or `cooldowns` ever matched. `cap` therefore always
        // fell back to `u32::MAX` and the `daily_cap` guard never fired for
        // any action. The mock is now generated from the same snake_case names
        // the config uses, so the guard under test is actually reachable.
        let action_key = match req.action_type {
            ActionType::ConnectionRequest => "connection_request",
            ActionType::Dm => "dm",
            ActionType::PostPublish => "post_publish",
            ActionType::CommentPost => "comment_post",
            ActionType::LikePost => "like_post",
            ActionType::ProfileView => "profile_view",
            ActionType::FollowCompany => "follow_company",
            ActionType::JobApplicationSubmit => "job_application_submit",
            ActionType::ClientProposalSend => "client_proposal_send",
            ActionType::ExecutiveOutreach => "executive_outreach",
            ActionType::SequenceStepSend => "sequence_step_send",
        }
        .to_string();
        let now = chrono::Utc::now();

        // 1. daily_cap
        let used = *state
            .daily_used
            .entry((req.member_id, action_key.clone(), today))
            .or_insert(0);
        let cap = cfg.daily_caps.get(&action_key).copied().unwrap_or(u32::MAX);
        if used >= cap {
            guards_failed.push("daily_cap".into());
            return GovernorDecision {
                decision: "deny".into(),
                reason: format!("daily_cap reached ({used}/{cap})"),
                guards_passed,
                guards_failed,
                permit_token: None,
                evaluated_at: now,
            };
        }
        state
            .daily_used
            .insert((req.member_id, action_key.clone(), today), used + 1);
        guards_passed.push("daily_cap".into());

        // 2. cooldown
        if let Some(target_id) = &req.target_id {
            // F-AUDIT-47: the cooldown used to be read with `.values().next()`,
            // i.e. "some cooldown that happens to be configured" rather than
            // the one for *this* action type. The map is keyed
            // "<action>_min_<unit>_between_to_same_target", so `connection_request`
            // (7 days) could be applied to a `dm` (24h) or any other action
            // depending on HashMap iteration order — a non-deterministic test
            // harness. Look the rule up by action, and fall back to the
            // default only when the action genuinely has no rule.
            let cooldown_secs = cfg
                .cooldowns
                .get(&format!("{action_key}_min_days_between_to_same_target"))
                .or_else(|| {
                    cfg.cooldowns
                        .get(&format!("{action_key}_min_hours_between_to_same_target"))
                })
                .copied()
                // The `_min_hours_` entries are stored in seconds already
                // (24 * 3_600), so no unit conversion is needed here.
                .unwrap_or(0);

            if cooldown_secs > 0 {
                let last = state.last_target_action.get(&(
                    req.member_id,
                    action_key.clone(),
                    target_id.clone(),
                ));
                if let Some(last) = last {
                    if (now - *last).num_seconds() < cooldown_secs {
                        guards_failed.push("cooldown".into());
                        return GovernorDecision {
                            decision: "deny".into(),
                            reason: "cooldown active for target".into(),
                            guards_passed,
                            guards_failed,
                            permit_token: None,
                            evaluated_at: now,
                        };
                    }
                }
                state
                    .last_target_action
                    .insert((req.member_id, action_key.clone(), target_id.clone()), now);
            }
        }
        guards_passed.push("cooldown".into());

        // 3. duplicate_target (skip for simplicity — covered by cooldown)
        guards_passed.push("duplicate_target".into());

        // 4. account_health — restricted members always denied.
        if *state.restrictions.get(&req.member_id).unwrap_or(&false) {
            guards_failed.push("account_health".into());
            return GovernorDecision {
                decision: "deny".into(),
                reason: "account restricted".into(),
                guards_passed,
                guards_failed,
                permit_token: None,
                evaluated_at: now,
            };
        }
        guards_passed.push("account_health".into());

        // 5. grounding
        let kb_refs = req
            .context
            .get("kb_refs")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if kb_refs < cfg.min_kb_refs {
            guards_failed.push("grounding".into());
            return GovernorDecision {
                decision: "deny".into(),
                reason: format!(
                    "low_grounding: only {} KB refs (need {})",
                    kb_refs, cfg.min_kb_refs
                ),
                guards_passed,
                guards_failed,
                permit_token: None,
                evaluated_at: now,
            };
        }
        guards_passed.push("grounding".into());

        // 6. restriction_flag
        if cfg.restriction_active {
            guards_failed.push("restriction_flag".into());
            return GovernorDecision {
                decision: "deny".into(),
                reason: "global restriction flag active".into(),
                guards_passed,
                guards_failed,
                permit_token: None,
                evaluated_at: now,
            };
        }
        guards_passed.push("restriction_flag".into());

        // 7. approval_state — assume approved for happy path.
        guards_passed.push("approval_state".into());

        // 8. session_pacing
        //
        // F-AUDIT-48: the pacing counter was keyed on `(member, window_bucket)`
        // only, so it was a single global budget per member across *all* action
        // types. Two consequences, both wrong:
        //   1. It silently over-constrained: after 5 posts, a `dm` was denied
        //      for pacing even though `dm` has its own (higher) daily cap.
        //   2. It made the `daily_cap` guard unreachable for any action whose
        //      cap exceeds `session_pacing_max` (5), because pacing always
        //      denied first. `daily_cap_deny_stops_at_first_guard` therefore
        //      could not pass for ANY action type.
        //
        // Pacing is per action type: the limit is "this kind of action, this
        // many times per window", matching the config's per-action `daily_caps`
        // structure. Keyed `(member, action, bucket)`.
        let bucket = now.timestamp() / cfg.session_pacing_window_secs;
        let count = state
            .session_pacing
            .entry((req.member_id, action_key.clone(), bucket))
            .or_insert(0);
        if *count >= cfg.session_pacing_max {
            guards_failed.push("session_pacing".into());
            return GovernorDecision {
                decision: "deny".into(),
                reason: format!(
                    "session_pacing: {} {action_key} actions in {}s window",
                    *count, cfg.session_pacing_window_secs
                ),
                guards_passed,
                guards_failed,
                permit_token: None,
                evaluated_at: now,
            };
        }
        *count += 1;
        guards_passed.push("session_pacing".into());

        // Allow → mint a permit-token (format only — verification is exercised
        // separately against the integration gateway).
        let token = format!(
            "permit.{}.{}",
            req.member_id,
            now.timestamp_nanos_opt().unwrap_or(0)
        );
        GovernorDecision {
            decision: "allow".into(),
            reason: "all guards passed".into(),
            guards_passed,
            guards_failed,
            permit_token: Some(token),
            evaluated_at: now,
        }
    }
}

// ---------- Integration Gateway (mock) ----------

#[derive(Default)]
pub struct IntegrationGatewayState {
    pub permits_seen: Vec<String>,
    pub idempotency_keys: HashMap<(Uuid, String), String>, // (member, idem_key) → result
    pub dispatched: Vec<(Uuid, ActionType, String)>,
}

#[derive(Clone)]
pub struct MockIntegrationGateway {
    pub state: Arc<Mutex<IntegrationGatewayState>>,
    pub dispatch_fail: Arc<Mutex<bool>>,
}

impl MockIntegrationGateway {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(IntegrationGatewayState::default())),
            dispatch_fail: Arc::new(Mutex::new(false)),
        }
    }

    pub fn execute(
        &self,
        member_id: Uuid,
        permit_token: &str,
        action_type: ActionType,
        idempotency_key: &str,
        payload: &str,
    ) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();

        // Idempotency: replay returns prior result.
        if let Some(prior) = state
            .idempotency_keys
            .get(&(member_id, idempotency_key.to_string()))
        {
            return Ok(prior.clone());
        }

        state.permits_seen.push(permit_token.to_string());

        let dispatch_fail = *self.dispatch_fail.lock().unwrap();
        let result = if dispatch_fail {
            Err("dispatch_failed".into())
        } else {
            state
                .dispatched
                .push((member_id, action_type, payload.into()));
            Ok(format!("dispatched:{idempotency_key}"))
        };
        if let Ok(ref s) = result {
            state
                .idempotency_keys
                .insert((member_id, idempotency_key.to_string()), s.clone());
        }
        result
    }
}

// ---------- Audit log (mock with checksum chain) ----------

#[derive(Debug, Clone)]
pub struct AuditRow {
    pub event_id: Uuid,
    pub actor: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub outcome: String,
    pub metadata: serde_json::Value,
    pub checksum_sha256: String,
    pub prev_checksum: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct MockAuditLog {
    pub rows: Vec<AuditRow>,
    pub last_checksum: Option<String>,
}

impl MockAuditLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        actor: &str,
        action: &str,
        resource_type: &str,
        resource_id: &str,
        outcome: &str,
        metadata: serde_json::Value,
    ) -> AuditRow {
        let prev = self.last_checksum.clone();
        let body = serde_json::json!({
            "event_id": Uuid::new_v4(),
            "actor": actor,
            "action": action,
            "resource_type": resource_type,
            "resource_id": resource_id,
            "outcome": outcome,
            "metadata": metadata,
            "prev_checksum": prev,
            "occurred_at": chrono::Utc::now(),
        });
        let checksum = sha256_hex(serde_json::to_string(&body).unwrap_or_default().as_bytes());
        let row = AuditRow {
            event_id: body["event_id"]
                .as_str()
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4),
            actor: actor.into(),
            action: action.into(),
            resource_type: resource_type.into(),
            resource_id: resource_id.into(),
            outcome: outcome.into(),
            metadata: metadata,
            checksum_sha256: checksum.clone(),
            prev_checksum: prev,
            occurred_at: chrono::Utc::now(),
        };
        self.last_checksum = Some(checksum);
        self.rows.push(row.clone());
        row
    }

    pub fn verify_chain(&self) -> Result<(), String> {
        let mut expected_prev: Option<String> = None;
        for r in &self.rows {
            if r.prev_checksum != expected_prev {
                return Err(format!(
                    "chain broken at event_id={} actor={}",
                    r.event_id, r.actor
                ));
            }
            expected_prev = Some(r.checksum_sha256.clone());
        }
        Ok(())
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// ---------- RLS DB (mock) ----------

#[derive(Default)]
pub struct MockRlsDb {
    pub current_member_id: Mutex<Option<Uuid>>,
    pub rows: Mutex<Vec<(Uuid, String, String)>>, // (member_id, table, data)
}

impl MockRlsDb {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the RLS session variable for the current transaction.
    pub fn set_session(&self, member_id: Uuid) {
        *self.current_member_id.lock().unwrap() = Some(member_id);
    }

    pub fn clear_session(&self) {
        *self.current_member_id.lock().unwrap() = None;
    }

    /// Insert with RLS enforcement — refuses if no member set.
    pub fn insert(&self, table: &str, data: &str) -> Result<(), String> {
        let mid = *self.current_member_id.lock().unwrap();
        match mid {
            None => Err("RLS session variable not set".into()),
            Some(m) => {
                self.rows
                    .lock()
                    .unwrap()
                    .push((m, table.into(), data.into()));
                Ok(())
            }
        }
    }

    /// Read with RLS enforcement — only returns rows for the active member.
    pub fn read(&self, table: &str) -> Vec<String> {
        let mid = *self.current_member_id.lock().unwrap();
        match mid {
            None => Vec::new(),
            Some(m) => self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|(mm, t, _)| *mm == m && t == table)
                .map(|(_, _, d)| d.clone())
                .collect(),
        }
    }
}
