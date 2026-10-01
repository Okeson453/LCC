//! identity-svc event publishing + consuming.

use lcc_events::envelope::Envelope;
use lcc_events::topics::Topic;
use lcc_events::publisher::Publisher;

pub async fn publish(p: &Publisher, topic: Topic, payload: serde_json::Value, member_id: Option<String>) -> Result<(), lcc_events::publisher::PublisherError> {
    let env = Envelope::new(topic, "identity-svc".to_string(), uuid::Uuid::new_v4().to_string(), payload, member_id);
    let id = p.publish(&env).await?;
    tracing::debug!(?id, "event published");
    Ok(())
}
