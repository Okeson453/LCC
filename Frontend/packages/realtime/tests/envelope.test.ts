import { describe, expect, it } from 'vitest';
import { parseEnvelope, isPingFrame, isPongFrame } from '../src/envelope';

describe('envelope', () => {
  it('parses a valid envelope', () => {
    const raw = JSON.stringify({
      event_id: '0d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
      event_type: 'briefing.refresh',
      occurred_at: '2026-04-22T07:00:00Z',
      trace_id: '1d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
      member_id: '2d9b2a5b-c7e1-4f47-89e8-1a2b3c4d5e6f',
      payload: { date: '2026-04-22' },
    });
    const env = parseEnvelope(raw);
    expect(env.event_type).toBe('briefing.refresh');
    expect(env.payload.date).toBe('2026-04-22');
  });

  it('rejects malformed JSON', () => {
    expect(() => parseEnvelope('not-json')).toThrow(/Malformed envelope/);
  });

  it('rejects envelope missing fields', () => {
    expect(() => parseEnvelope(JSON.stringify({ event_type: 'briefing.refresh' }))).toThrow(/missing field/);
  });

  it('detects ping and pong frames', () => {
    expect(isPingFrame(JSON.stringify({ type: 'ping', ts: 1 }))).toBe(true);
    expect(isPongFrame(JSON.stringify({ type: 'pong', ts: 1 }))).toBe(true);
    expect(isPingFrame(JSON.stringify({ type: 'pong' }))).toBe(false);
  });
});
