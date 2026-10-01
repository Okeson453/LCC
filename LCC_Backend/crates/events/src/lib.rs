//! `lcc-events` — Event bus abstractions (envelope, publisher, consumer, topic constants).
//!
//! Phase 1-2 uses Redis Streams; Phase 3+ migrates to Kafka. The publisher
//! and consumer abstractions abstract over the underlying transport so the
//! migration is a configuration change, not a code change.

pub mod consumer;
pub mod envelope;
pub mod publisher;
pub mod topics;

pub use consumer::{Consumer, ConsumerError, ConsumerHandler};
pub use envelope::{
    Envelope, EventHeader, EventPayload, MemberCreatedEvent, ContentItemApprovedEvent,
    ContentItemStateChangedEvent, EngagementInboundReceivedEvent, OpportunityDiscoveredEvent,
    SequenceReplyDetectedEvent, SequenceStepDueEvent, ComplianceConfigActivatedEvent,
    ComplianceRestrictionDetectedEvent, AuditEvent, ApprovalDecidedEvent, KbRecordCreatedEvent,
};
pub use publisher::{Publisher, PublisherError};
pub use topics::Topic;
