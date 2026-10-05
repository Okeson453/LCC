import { describe, expect, it } from 'vitest';
import {
  APPROVALS_CHANNEL_PATH,
  BRIEFING_CHANNEL_PATH,
  COMPLIANCE_CHANNEL_PATH,
  ENGAGEMENT_CHANNEL_PATH,
  INTEGRATION_CHANNEL_PATH,
  SEQUENCE_CHANNEL_PATH,
} from '@lcc/realtime';

/**
 * These assertions target the API `@lcc/realtime` actually exports.
 *
 * The previous version imported `briefingChannel`, `approvalsChannel`, … — a
 * set of channel-name builders the package never had. What it does export per
 * channel is a `subscribe*Channel(client, listener)` helper and a
 * `*_CHANNEL_PATH` constant. The channel paths are the contract the backend
 * routes on (realtime-svc registers `/api/v1/ws/<channel>`), so asserting on
 * them is what actually guards the wiring.
 */
describe('channel paths', () => {
  it('exposes the five dashboard WS channels', () => {
    expect(BRIEFING_CHANNEL_PATH).toBe('/api/v1/ws/briefing');
    expect(APPROVALS_CHANNEL_PATH).toBe('/api/v1/ws/approvals');
    expect(ENGAGEMENT_CHANNEL_PATH).toBe('/api/v1/ws/engagement');
    expect(COMPLIANCE_CHANNEL_PATH).toBe('/api/v1/ws/compliance');
    expect(SEQUENCE_CHANNEL_PATH).toBe('/api/v1/ws/sequence');
  });

  it('exposes the Track B integration channel', () => {
    expect(INTEGRATION_CHANNEL_PATH).toBe('/api/v1/ws/integration');
  });

  it('keeps every channel under the canonical /api/v1/ws namespace', () => {
    for (const p of [
      BRIEFING_CHANNEL_PATH,
      APPROVALS_CHANNEL_PATH,
      ENGAGEMENT_CHANNEL_PATH,
      COMPLIANCE_CHANNEL_PATH,
      SEQUENCE_CHANNEL_PATH,
      INTEGRATION_CHANNEL_PATH,
    ]) {
      expect(p.startsWith('/api/v1/ws/')).toBe(true);
    }
  });
});
