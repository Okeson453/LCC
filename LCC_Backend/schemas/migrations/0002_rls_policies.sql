-- 0002_rls_policies.sql
-- Row-Level Security policies for per-member isolation.
-- Per Backend Design Concept §39: every transactional table has RLS
-- keyed on the session variable `app.current_member_id`.

BEGIN;

-- All member-scoped tables share the pattern:
--   USING (member_id::text = current_setting('app.current_member_id', true))
--   WITH CHECK (...)

-- Helper to add RLS on a table.
--
-- F-AUDIT-33: the previous body emitted a *bare* identifier:
--     EXECUTE format('ALTER TABLE %I ...', p_table);
-- `%I` quotes the name but does NOT schema-qualify it, so Postgres resolved it
-- through `search_path`, which defaults to `"$user", public`. No
-- `search_path` is set anywhere in this repository, so every call such as
-- `SELECT lcc.attach_member_rls('members')` in migration 0003 raised
--     ERROR: relation "members" does not exist
-- and migrations 0003+ could not complete. The database schema could not be
-- built at all.
--
-- The fix is to schema-qualify explicitly. `%I` is still used for the bare
-- name so an injection attempt like `attach_member_rls('x; DROP TABLE y')`
-- is quoted as a single identifier rather than executed.
--
-- F-AUDIT-34: the policy template also hardcoded a bare `member_id` column.
-- That is correct for the ~30 member-scoped tables, but `lcc.members` is
-- itself keyed on `id`, not `member_id`, so `attach_member_rls('members')`
-- created a policy referencing a non-existent column and every INSERT/UPDATE
-- on that table failed at `WITH CHECK`. `p_member_column` makes the key
-- column explicit and defaults to the common case.
CREATE OR REPLACE FUNCTION lcc.attach_member_rls(
    p_table text,
    p_member_column text DEFAULT 'member_id'
) RETURNS void AS $$
DECLARE
  pol_name text;
  qualified text := format('lcc.%I', p_table);
BEGIN
  pol_name := 'member_isolation_' || p_table;
  EXECUTE format('ALTER TABLE %s ENABLE ROW LEVEL SECURITY', qualified);
  EXECUTE format('ALTER TABLE %s FORCE ROW LEVEL SECURITY', qualified);
  EXECUTE format('DROP POLICY IF EXISTS %I ON %s', pol_name, qualified);
  EXECUTE format(
    'CREATE POLICY %I ON %s USING (%I::text = current_setting(''app.current_member_id'', true)) WITH CHECK (%I::text = current_setting(''app.current_member_id'', true))',
    pol_name, qualified, p_member_column, p_member_column
  );
END;
$$ LANGUAGE plpgsql;

-- A read-only role used by analytics services.
GRANT USAGE ON SCHEMA lcc TO lcc_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA lcc GRANT SELECT ON TABLES TO lcc_readonly;

COMMIT;
