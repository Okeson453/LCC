//! identity-svc event publishing + consuming.

use lcc_events::envelope::{Envelope, EventPayload};
use lcc_events::topics::Topic;
use lcc_events::publisher::Publisher;

pub async fn publish(
    p: &Publisher,
    topic: Topic,
    payload: EventPayload,
    member_id: Option<String>,
) -> Result<(), lcc_events::publisher::PublisherError> {
    let mut env = Envelope::new(topic, "identity-svc".to_string(), uuid::Uuid::new_v4().to_string(), payload);
    // `member_id` scopes the event for RLS; system events carry none.
    if let Some(member_id) = member_id {
        env = env.with_member(member_id);
    }
    let id = p.publish(&env).await?;
    tracing::debug!(?id, "event published");
    Ok(())
}
