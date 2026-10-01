-- 9999_seed_dev.sql
-- Development-only seed data. NEVER applied in production.

BEGIN;

-- Seed a single member for local development.
INSERT INTO lcc.members (id, linkedin_id, email, display_name, role)
VALUES ('00000000-0000-0000-0000-000000000001', 'dev-linkedin-id-1', 'dev@lcc.local', 'Dev Member', 'member')
ON CONFLICT DO NOTHING;

-- Seed a compliance config.
INSERT INTO lcc.compliance_config_versions (version, is_active, config, activated_at, activated_by, two_reviewer_signed_by)
VALUES (
    'ccfg-2025-01-01-rc1',
    TRUE,
    jsonb_build_object(
        'h_c', jsonb_build_object(
            'weights', jsonb_build_object(
                'response_rate', 0.30, 'acceptance_rate', 0.20,
                'error_rate_inverted', 0.20, 'restriction_inverted', 0.15,
                'engagement_quality', 0.15
            ),
            'warmup_threshold', 0.85, 'standard_threshold', 0.65,
            'reserve_fraction', 0.10
        ),
        'ab_d', jsonb_build_object('multiplier_floor', 0.5, 'multiplier_ceiling', 2.0),
        'rho', jsonb_build_object('labeled_send_threshold', 200, 'vip_boost', 1.5),
        'daily_caps', jsonb_build_object(
            'connection_request', 25, 'dm', 40, 'post_publish', 2,
            'sequence_step_send', 20, 'job_application_submit', 8,
            'client_proposal_send', 5, 'executive_outreach', 3
        ),
        'cooldowns', jsonb_build_object(
            'connection_request_min_days_between_to_same_target', 90,
            'dm_min_hours_between_to_same_target', 24
        )
    ),
    NOW(),
    'dev-bootstrap',
    ARRAY['dev-reviewer-1', 'dev-reviewer-2']
)
ON CONFLICT DO NOTHING;

-- Seed a few tags.
INSERT INTO lcc.tags (member_id, name, color) VALUES
    ('00000000-0000-0000-0000-000000000001', 'prospect', '#3B82F6'),
    ('00000000-0000-0000-0000-000000000001', 'partner', '#10B981'),
    ('00000000-0000-0000-0000-000000000001', 'mentor', '#F59E0B')
ON CONFLICT DO NOTHING;

COMMIT;
