/**
 * Frontend → contract path conformance.
 *
 * Every literal path the web dashboard sends must exist in one of the two
 * contracts: the canonical REST contract or the canonical realtime contract.
 * This is the frontend half of the integration check — the gateway half lives
 * in `Backend/tests/contract/gateway_contract_conformance.rs`, which asserts
 * the routing table covers the contract.
 *
 * The audit found the frontend and backend describing *different* APIs
 * (`LCC_AUDIT_REPORT.md` §3.3: "Zero overlap between the 49 endpoints the
 * frontend calls and anything the backend implements"). Nothing in CI detected
 * it, because the frontend types were generated from a superseded 55-path
 * document rather than from the canonical contract.
 *
 * `KNOWN_DIVERGENCES` records the paths that are genuinely unreconciled today.
 * They are listed rather than hidden so the list can only shrink: a NEW
 * frontend path that matches no contract path fails the suite immediately.
 */

import { describe, expect, it } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { API_PREFIX, API_PATHS, SSE_CHANNELS, WS_CHANNELS } from '@/lib/api/config';

const REPO_ROOT = join(__dirname, '../../../../..');
const CANONICAL = join(REPO_ROOT, 'Contract/openapi/lcc-api-canonical.yaml');
const REALTIME = join(REPO_ROOT, 'Contract/realtime/lcc-realtime-contract.yaml');

/** Infra operations live at the host root, not under `/api/v1`. */
const INFRA_PATHS = new Set(['/healthz', '/readyz', '/metrics']);

/** `servers[0].url` carries the namespace; path keys are relative to it. */
function serverBasePath(openapi: string): string {
  let inServers = false;
  for (const line of openapi.split('\n')) {
    if (line.startsWith('servers:')) {
      inServers = true;
      continue;
    }
    if (!inServers) continue;
    if (line && !line.startsWith(' ') && line.trim()) break;
    const url = line.trim().replace(/^- url:\s*/, '');
    if (url !== line.trim() || line.trim().startsWith('- url:')) {
      const afterScheme = url.indexOf('://');
      if (afterScheme === -1) return '';
      const slash = url.indexOf('/', afterScheme + 3);
      return slash === -1 ? '' : url.slice(slash).replace(/\/$/, '');
    }
  }
  return '';
}

/** Every path either contract declares, with `{param}` normalised to `{}`. */
function contractPaths(): Set<string> {
  const openapi = readFileSync(CANONICAL, 'utf8');
  const base = serverBasePath(openapi);
  const out = new Set<string>();

  for (const line of openapi.split('\n')) {
    const m = /^ {2}(\/\S+):\s*$/.exec(line);
    if (!m) continue;
    const p = m[1];
    if (p === undefined) continue;
    out.add(normalise(INFRA_PATHS.has(p) ? p : `${base}${p}`));
  }

  const realtime = readFileSync(REALTIME, 'utf8');
  for (const m of realtime.matchAll(/url_(?:ws|sse):\s*(\S+)/g)) {
    const p = m[1];
    if (p !== undefined) out.add(normalise(p));
  }
  return out;
}

/** The probe value used when calling an `API_PATHS` arrow function. */
const PROBE_ID = 'probe-id';

/**
 * Reduce a concrete path to its shape: every id becomes `{}`, so
 * `/api/v1/contacts/probe-id` and `/api/v1/contacts/${id}` compare equal
 * against the contract's `/api/v1/contacts/{contactId}`.
 */
function normalise(p: string): string {
  // Strip the query string, then collapse every id to `{}` so
  // `/api/v1/contacts/probe-id`, `/api/v1/contacts/${id}` and the contract's
  // `/api/v1/contacts/{contactId}` all compare equal.
  const withoutQuery = p.split('?')[0] ?? p;
  const withoutTemplates = withoutQuery
    .replace(/\$\{[^}]+\}/g, '{}')
    .replace(/\{[^}]+\}/g, '{}');
  return withoutTemplates
    .split('/')
    .map((seg) => (seg === PROBE_ID ? '{}' : seg))
    .join('/');
}

/** Expand `API_PATHS` (nested, may contain arrow functions) into path strings. */
function declaredFrontendPaths(): string[] {
  const found = new Set<string>();
  const walk = (node: unknown): void => {
    if (typeof node === 'string') {
      if (node.startsWith(API_PREFIX) || INFRA_PATHS.has(node)) found.add(node);
      return;
    }
    if (typeof node === 'function') {
      // e.g. `(id) => `${API_PREFIX}/approvals/${id}`` — call with a probe id.
      try {
        walk((node as (v: string) => string)(PROBE_ID));
      } catch {
        /* not a path builder; ignore */
      }
      return;
    }
    if (node && typeof node === 'object') {
      Object.values(node as Record<string, unknown>).forEach(walk);
    }
  };
  walk(API_PATHS);
  return [...found];
}

/** Literal paths written inline in `src/lib/api/*.ts` (outside API_PATHS). */
function inlineFrontendPaths(): string[] {
  const dir = join(__dirname, '../../src/lib/api');
  const out = new Set<string>();
  for (const file of readdirSync(dir).filter((f) => f.endsWith('.ts'))) {
    const src = readFileSync(join(dir, file), 'utf8');
    for (const m of src.matchAll(/apiFetch<[^>]*>\(\s*[`'"]([^`'"]+)[`'"]/g)) {
      if (m[1] !== undefined) out.add(m[1]);
    }
    for (const m of src.matchAll(/\$\{API_PREFIX\}(\/[\w${}./-]+)/g)) {
      if (m[1] !== undefined) out.add(`${API_PREFIX}${m[1]}`);
    }
  }
  return [...out];
}

/**
 * Frontend paths with no counterpart in either contract today.
 *
 * Two distinct causes, both tracked in the audit:
 *  - the contract nests the domain surface under `/api/v1/members/{memberId}/…`
 *    while the client calls the flat `/api/v1/<domain>/…` form;
 *  - some paths exist in neither (e.g. `/api/v1/admin/evaluate`).
 *
 * Listed explicitly so a newly-introduced divergence fails the suite. Remove
 * entries as call sites are migrated to the canonical shape.
 */
const KNOWN_DIVERGENCES = new Set([
  '/api/v1/admin/compliance/config-versions/{}/review',
  '/api/v1/admin/evaluate',
  '/api/v1/analytics/account-health',
  '/api/v1/analytics/content',
  '/api/v1/analytics/dashboard',
  '/api/v1/analytics/digest/monthly',
  '/api/v1/analytics/digest/weekly',
  '/api/v1/analytics/funnel/client',
  '/api/v1/analytics/funnel/job',
  '/api/v1/analytics/network',
  '/api/v1/analytics/outreach',
  '/api/v1/analytics/profile',
  '/api/v1/analytics/time-series',
  '/api/v1/approvals',
  '/api/v1/approvals/bulk-decide',
  '/api/v1/approvals/{}',
  '/api/v1/approvals/{}/decide',
  '/api/v1/audit/events',
  '/api/v1/audit/events/{}',
  '/api/v1/companies',
  '/api/v1/companies/staleness',
  '/api/v1/contacts',
  '/api/v1/contacts/{}',
  '/api/v1/contacts/{}/company',
  '/api/v1/contacts/{}/interactions',
  '/api/v1/content/calendar',
  '/api/v1/content/items',
  '/api/v1/content/items/compose',
  '/api/v1/content/items/{}',
  '/api/v1/content/items/{}/quality-check',
  '/api/v1/content/items/{}/schedule',
  '/api/v1/content/items/{}/transition',
  '/api/v1/engagement/inbox',
  '/api/v1/engagement/inbox/{}/read',
  '/api/v1/engagement/queue',
  '/api/v1/engagement/tasks',
  '/api/v1/engagement/tasks/{}/complete',
  '/api/v1/engagement/tasks/{}/dismiss',
  '/api/v1/engagement/tasks/{}/draft',
  '/api/v1/kb/records',
  '/api/v1/kb/records/{}',
  '/api/v1/kb/records/{}/embedding-status',
  '/api/v1/kb/records/{}/reembed',
  '/api/v1/members/{}',
  '/api/v1/members/{}/briefing/refresh',
  '/api/v1/opportunities',
  '/api/v1/opportunities/applications',
  '/api/v1/opportunities/discover',
  '/api/v1/opportunities/proposals',
  '/api/v1/opportunities/proposals/draft',
  '/api/v1/opportunities/{}',
  '/api/v1/opportunities/{}/apply',
  '/api/v1/opportunities/{}/qualify',
  '/api/v1/outreach/templates',
  '/api/v1/profile/me',
  '/api/v1/profile/me/audit',
  '/api/v1/profile/me/consent/{}',
  '/api/v1/profile/me/edit-drafts',
  '/api/v1/profile/me/edit-drafts/{}/decide',
  '/api/v1/profile/me/strength-history',
  '/api/v1/sequences',
  '/api/v1/sequences/steps/{}/reply',
  '/api/v1/sequences/steps/{}/sent',
  '/api/v1/sequences/{}',
  '/api/v1/sequences/{}/pause',
  '/api/v1/sequences/{}/resume',
  '/api/v1/sequences/{}/steps',
]);

describe('frontend ↔ contract conformance', () => {
  it('declares the canonical /api/v1 namespace', () => {
    expect(API_PREFIX).toBe('/api/v1');
    expect(serverBasePath(readFileSync(CANONICAL, 'utf8'))).toBe('/api/v1');
  });

  it('routes every declared frontend path to a contract path', () => {
    const contract = contractPaths();
    const all = [...declaredFrontendPaths(), ...inlineFrontendPaths()];
    const unexpected = all
      .map(normalise)
      .filter((p) => !contract.has(p))
      .filter((p) => !KNOWN_DIVERGENCES.has(p) && ![...KNOWN_DIVERGENCES].some((k) => normalise(k) === p));

    expect(
      unexpected,
      `frontend calls paths that no contract declares:\n  ${unexpected.join('\n  ')}\n` +
        'Add them to the canonical contract, or to KNOWN_DIVERGENCES if they are tracked debt.',
    ).toEqual([]);
  });

  it('uses the realtime contract for every WS and SSE channel', () => {
    const contract = contractPaths();
    for (const ch of [...Object.values(WS_CHANNELS), ...Object.values(SSE_CHANNELS)]) {
      expect(contract, `${ch} is not declared by the realtime contract`).toContain(ch);
    }
  });

  it('keeps the known-divergence list honest', () => {
    // An entry that now matches the contract is stale and should be removed,
    // which keeps the list an accurate measure of remaining debt.
    const contract = contractPaths();
    const stale = [...KNOWN_DIVERGENCES].filter((p) => contract.has(normalise(p)));
    expect(
      stale,
      `these entries now match the contract — remove them from KNOWN_DIVERGENCES:\n  ${stale.join('\n  ')}`,
    ).toEqual([]);
  });
});
