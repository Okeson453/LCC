import { describe, expect, it } from 'vitest';
import { ReconnectionState } from '../src/utils/reconnection-state';

describe('ReconnectionState', () => {
  it('starts disconnected', () => {
    const s = new ReconnectionState();
    expect(s.current.status).toBe('disconnected');
    expect(s.current.transport).toBe('none');
  });

  it('notifies subscribers on update', () => {
    const s = new ReconnectionState();
    const calls: number[] = [];
    s.subscribe((st) => calls.push(st.attempt));
    s.update({ attempt: 1 });
    s.update({ attempt: 2 });
    expect(calls).toEqual([0, 1, 2]);
  });

  it('unsubscribe stops notifications', () => {
    const s = new ReconnectionState();
    let calls = 0;
    const unsub = s.subscribe(() => calls++);
    s.update({ status: 'connected' });
    expect(calls).toBe(2); // initial + update
    unsub();
    s.update({ status: 'reconnecting' });
    expect(calls).toBe(2);
  });

  it('returns immutable snapshots', () => {
    const s = new ReconnectionState();
    const snap1 = s.current;
    snap1.attempt = 999; // mutate the returned object
    expect(s.current.attempt).toBe(0);
  });
});
