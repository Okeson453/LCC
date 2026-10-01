-- 0001_init_extensions_and_roles.sql
-- Initial setup: extensions, schemas, and role separation.

BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "citext";

CREATE SCHEMA IF NOT EXISTS lcc;
CREATE SCHEMA IF NOT EXISTS lcc_audit;

-- Application role — full CRUD within the lcc schema.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_app') THEN
    CREATE ROLE lcc_app LOGIN PASSWORD 'dev_lcc_app';
  END IF;
END$$;

-- Audit role — INSERT-only on audit tables (no UPDATE, no DELETE).
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_audit_writer') THEN
    CREATE ROLE lcc_audit_writer LOGIN PASSWORD 'dev_audit_writer';
  END IF;
END$$;

-- Read-only role for analytics.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lcc_readonly') THEN
    CREATE ROLE lcc_readonly LOGIN PASSWORD 'dev_readonly';
  END IF;
END$$;

-- Per-tenant isolation: Row-Level Security will be enabled in 0002_rls_policies.sql.

COMMIT;
