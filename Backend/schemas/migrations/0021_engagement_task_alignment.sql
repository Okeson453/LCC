-- 0021_engagement_task_alignment.sql
-- Closes the last gap between `lcc.engagement_replies` and the canonical
-- `EngagementTask` shape.
--
-- WHY THIS MIGRATION EXISTS
-- -------------------------
-- Migration 0018 already began aligning `lcc.engagement_replies` to the
-- canonical EngagementTask ("0018 ... engagement_replies - confirm shape aligns
-- with canonical EngagementTask") and added `priority_score`, `due_at`,
-- `completed_at`, `target_post_id` and `version`. Its header asserts that 0007
-- "already declares: id, member_id, inbound_message_id (nullable), kind,
-- draft_body, status" — that assertion is FALSE. 0007 declares only
--   id, member_id, inbound_message_id, content_item_id, status, created_at
-- so `kind` and `draft_body` were never created and have been missing ever since.
--
-- Three independent authorities require the same three columns, and no existing
-- column can serve any of them:
--
--   column        Backend Design Concept §11.12   Contract/openapi
--                 (lcc-api-canonical.yaml)         EngagementTask
--                 proto/lcc/v1/engagement
--   action_type   NOT NULL CHECK (reply,comment,  required: true
--                 like,congratulate,                enum [reply,comment,like,
--                 connection_accept,follow_up)     congratulate,
--                                                    connection_accept,follow_up]
--                                                    EngagementActionType enum
--   draft_body    draft_body TEXT                  draft_body: string|null
--                                                    proto: string draft_body
--   contact_id    contact_id UUID REFERENCES       contact_id: uuid|null
--                 contact(id)                      proto: string contact_id
--
-- `Contract/docs/endpoint_contract_matrix.md:376` records §11.12
-- `lcc.engagement_replies.draft_body` as "present". It was not. This migration
-- makes that record true.
--
-- `action_type` has no possible derivation. `lcc.inbound_messages.kind` is a
-- different concept (how the message ARRIVED: comment/dm/mention/
-- connection_request_inbound/post_reaction) and mapping it onto an outbound
-- action would invent a business rule, so no such mapping is made here.
--
-- `contact_id` IS derivable for reply-shaped rows (via the
-- engagement_replies -> inbound_messages FK) and is backfilled that way below.
-- It is still added as a real column because the daily-ritual queue - the
-- primary creation path per design §18.4 - has no inbound message at all, so
-- the derivation alone would silently drop the contact the caller supplied.
--
-- `status` was declared TEXT with no CHECK in 0007, so a service could write
-- any string at all. The vocabulary below is design §11.12's CHECK list, which
-- is identical to the OpenAPI `EngagementTask.status` enum and to the proto
-- `EngagementTaskStatus` enum. This is the same fail-closed reasoning as 0015:
-- a value outside the vocabulary is a defect, and it should fail at the column
-- rather than surface as wrong data under a 200.
--
-- NOT ADDED, deliberately:
--   * `updated_at` - in neither the contract nor design §11.12. `version` is the
--     optimistic-concurrency column the contract does declare.
--   * `draft_pins` - in neither the contract, the design, nor the proto.

BEGIN;

-- -----------------------------------------------------------------------------
-- action_type
-- -----------------------------------------------------------------------------
ALTER TABLE lcc.engagement_replies
    ADD COLUMN IF NOT EXISTS action_type TEXT;

-- Backfill. Every row that exists today is a row in `engagement_replies` -
-- the table is the reply/task queue and 0007's own header calls it "engagement
-- replies" - so 'reply' is the only action its existing rows can represent.
-- The service always supplies action_type explicitly on INSERT; this default
-- only covers rows that predate the column.
UPDATE lcc.engagement_replies
   SET action_type = 'reply'
 WHERE action_type IS NULL;

ALTER TABLE lcc.engagement_replies
    ALTER COLUMN action_type SET DEFAULT 'reply',
    ALTER COLUMN action_type SET NOT NULL;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'engagement_replies_action_type_check'
       AND conrelid = 'lcc.engagement_replies'::regclass
  ) THEN
    ALTER TABLE lcc.engagement_replies
      ADD CONSTRAINT engagement_replies_action_type_check
      CHECK (action_type IN ('reply','comment','like','congratulate',
                             'connection_accept','follow_up'));
  END IF;
END$$;

-- -----------------------------------------------------------------------------
-- draft_body
-- -----------------------------------------------------------------------------
ALTER TABLE lcc.engagement_replies
    ADD COLUMN IF NOT EXISTS draft_body TEXT;

-- -----------------------------------------------------------------------------
-- contact_id (+ FK, + evidence-based backfill)
-- -----------------------------------------------------------------------------
ALTER TABLE lcc.engagement_replies
    ADD COLUMN IF NOT EXISTS contact_id UUID;

-- Backfill from the authoritative relationship already in the schema:
-- engagement_replies.inbound_message_id -> inbound_messages.contact_id (0007).
-- No value is invented: a row without an inbound message keeps NULL, which the
-- contract already permits (`contact_id: {nullable: true}`).
UPDATE lcc.engagement_replies er
   SET contact_id = im.contact_id
  FROM lcc.inbound_messages im
 WHERE er.inbound_message_id = im.id
   AND er.contact_id IS NULL
   AND im.contact_id IS NOT NULL;

DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'engagement_replies_contact_id_fkey'
       AND conrelid = 'lcc.engagement_replies'::regclass
  ) THEN
    ALTER TABLE lcc.engagement_replies
      ADD CONSTRAINT engagement_replies_contact_id_fkey
      FOREIGN KEY (contact_id) REFERENCES lcc.contacts(id) ON DELETE SET NULL;
  END IF;
END$$;

CREATE INDEX IF NOT EXISTS idx_engagement_replies_contact
    ON lcc.engagement_replies (contact_id)
    WHERE contact_id IS NOT NULL;

-- -----------------------------------------------------------------------------
-- status vocabulary (design §11.12 == OpenAPI enum == proto enum)
-- -----------------------------------------------------------------------------
-- Applied only when no row violates it, so a database holding a value outside
-- the canonical vocabulary keeps serving and reports the offending rows rather
-- than having the migration abort. The application is fixed to the same
-- vocabulary, so it cannot reintroduce a bad value.
DO $$
DECLARE
    bad TEXT;
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
         WHERE conname = 'engagement_replies_status_check'
           AND conrelid = 'lcc.engagement_replies'::regclass
    ) THEN
        RETURN;
    END IF;

    SELECT string_agg(DISTINCT status, ', ') INTO bad
      FROM lcc.engagement_replies
     WHERE status NOT IN ('queued','drafted','approved','sent','dismissed','expired');

    IF bad IS NULL THEN
        ALTER TABLE lcc.engagement_replies
          ADD CONSTRAINT engagement_replies_status_check
          CHECK (status IN ('queued','drafted','approved','sent','dismissed','expired'));
    ELSE
        RAISE WARNING
            'engagement_replies_status_check NOT added: pre-existing rows use '
            'status values outside the canonical vocabulary (%). Repair those '
            'rows before relying on the constraint.', bad;
    END IF;
END$$;

-- Supports the queue listing, which orders by due_at within a member.
CREATE INDEX IF NOT EXISTS idx_engagement_replies_member_due
    ON lcc.engagement_replies (member_id, due_at NULLS LAST, created_at DESC);

COMMIT;
