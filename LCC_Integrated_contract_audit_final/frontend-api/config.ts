/**
 * API client config — base URL, timeouts, retries.
 *
 * MIGRATION NOTE: All paths are now prefixed with `/api/v1/` per the
 * canonical LCC OpenAPI contract (see `contract_audit/openapi/lcc-api-canonical.yaml`).
 * The gateway enforces this namespace and removes the legacy `/v1/<svc>_svc/...`
 * shape entirely.
 */

const envBase = (typeof process !== 'undefined' && process.env.NEXT_PUBLIC_API_BASE) || '';

/** Canonical API prefix. */
export const API_PREFIX = '/api/v1';

export const API_BASE = envBase || '';
export const WS_BASE =
  (typeof process !== 'undefined' && process.env.NEXT_PUBLIC_WS_BASE) || '';

export const API_TIMEOUT_MS = 30_000;
export const API_MAX_RETRIES = 2;

/**
 * Canonical API paths. Single source of truth — never inline a path
 * outside this constant.
 */
export const API_PATHS = {
  auth: {
    linkedinStart: `${API_PREFIX}/auth/linkedin/start`,
    linkedinCallback: `${API_PREFIX}/auth/linkedin/callback`,
    refresh: `${API_PREFIX}/auth/refresh`,
    logout: `${API_PREFIX}/auth/logout`,
  },
  members: {
    me: `${API_PREFIX}/members/me`,
    settings: `${API_PREFIX}/members/me/settings`,
    get: (id: string) => `${API_PREFIX}/members/${id}`,
    export: (id: string) => `${API_PREFIX}/members/${id}/export`,
  },
  profile: {
    me: `${API_PREFIX}/profile/me`,
    editDrafts: `${API_PREFIX}/profile/me/edit-drafts`,
    editDraftDecide: (draftId: string) =>
      `${API_PREFIX}/profile/me/edit-drafts/${draftId}/decide`,
    consent: (kind: string) => `${API_PREFIX}/profile/me/consent/${kind}`,
  },
  content: {
    items: `${API_PREFIX}/content/items`,
    item: (id: string) => `${API_PREFIX}/content/items/${id}`,
    transition: (id: string) => `${API_PREFIX}/content/items/${id}/transition`,
    qualityCheck: (id: string) => `${API_PREFIX}/content/items/${id}/quality-check`,
    schedule: (id: string) => `${API_PREFIX}/content/items/${id}/schedule`,
  },
  contacts: {
    list: `${API_PREFIX}/contacts`,
    item: (id: string) => `${API_PREFIX}/contacts/${id}`,
    interactions: (id: string) => `${API_PREFIX}/contacts/${id}/interactions`,
    company: (id: string) => `${API_PREFIX}/contacts/${id}/company`,
    companies: `${API_PREFIX}/companies`,
    staleness: `${API_PREFIX}/companies/staleness`,
  },
  opportunities: {
    list: `${API_PREFIX}/opportunities`,
    qualify: (id: string) => `${API_PREFIX}/opportunities/${id}/qualify`,
    apply: (id: string) => `${API_PREFIX}/opportunities/${id}/apply`,
    applications: `${API_PREFIX}/opportunities/applications`,
    proposals: `${API_PREFIX}/opportunities/proposals`,
  },
  sequences: {
    list: `${API_PREFIX}/sequences`,
    steps: (id: string) => `${API_PREFIX}/sequences/${id}/steps`,
    pause: (id: string) => `${API_PREFIX}/sequences/${id}/pause`,
    resume: (id: string) => `${API_PREFIX}/sequences/${id}/resume`,
    stepSent: (stepId: string) => `${API_PREFIX}/sequences/steps/${stepId}/sent`,
    stepReply: (stepId: string) => `${API_PREFIX}/sequences/steps/${stepId}/reply`,
    templates: `${API_PREFIX}/outreach/templates`,
  },
  engagement: {
    inbox: `${API_PREFIX}/engagement/inbox`,
    markRead: (id: string) => `${API_PREFIX}/engagement/inbox/${id}/read`,
    queue: `${API_PREFIX}/engagement/queue`,
    tasks: `${API_PREFIX}/engagement/tasks`,
    draft: (id: string) => `${API_PREFIX}/engagement/tasks/${id}/draft`,
    complete: (id: string) => `${API_PREFIX}/engagement/tasks/${id}/complete`,
    dismiss: (id: string) => `${API_PREFIX}/engagement/tasks/${id}/dismiss`,
  },
  approvals: {
    list: `${API_PREFIX}/approvals`,
    item: (id: string) => `${API_PREFIX}/approvals/${id}`,
    decide: (id: string) => `${API_PREFIX}/approvals/${id}/decide`,
    bulkDecide: `${API_PREFIX}/approvals/bulk-decide`,
  },
  admin: {
    complianceConfigVersions: `${API_PREFIX}/admin/compliance/config-versions`,
    reviewConfig: (id: string) =>
      `${API_PREFIX}/admin/compliance/config-versions/${id}/review`,
    activateConfig: (id: string) =>
      `${API_PREFIX}/admin/compliance/config-versions/${id}/activate`,
    listConfigVersions: `${API_PREFIX}/admin/compliance/config-versions`,
    evaluate: `${API_PREFIX}/admin/evaluate`,
  },
  kb: {
    records: `${API_PREFIX}/kb/records`,
    record: (id: string) => `${API_PREFIX}/kb/records/${id}`,
    reembed: (id: string) => `${API_PREFIX}/kb/records/${id}/reembed`,
    embeddingStatus: (id: string) =>
      `${API_PREFIX}/kb/records/${id}/embedding-status`,
  },
  analytics: {
    dashboard: `${API_PREFIX}/analytics/dashboard`,
    timeSeries: `${API_PREFIX}/analytics/time-series`,
  },
  audit: {
    events: `${API_PREFIX}/audit/events`,
    event: (id: string) => `${API_PREFIX}/audit/events/${id}`,
  },
  briefing: {
    today: (memberId: string) =>
      `${API_PREFIX}/members/${memberId}/briefing/today`,
    refresh: (memberId: string) =>
      `${API_PREFIX}/members/${memberId}/briefing/refresh`,
  },
} as const;

/** Realtime WS channels — paths under `/api/v1/ws/...`. */
export const WS_CHANNELS = {
  briefing: `${API_PREFIX}/ws/briefing`,
  approvals: `${API_PREFIX}/ws/approvals`,
  engagement: `${API_PREFIX}/ws/engagement`,
  compliance: `${API_PREFIX}/ws/compliance`,
  sequence: `${API_PREFIX}/ws/sequence`,
} as const;

/** SSE fallback paths. */
export const SSE_CHANNELS = {
  briefing: `${API_PREFIX}/sse/briefing`,
  approvals: `${API_PREFIX}/sse/approvals`,
  engagement: `${API_PREFIX}/sse/engagement`,
  compliance: `${API_PREFIX}/sse/compliance`,
  sequence: `${API_PREFIX}/sse/sequence`,
} as const;
