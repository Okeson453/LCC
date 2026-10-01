import { describe, expect, it } from 'vitest';
import { redactAccessTokens, redactPII } from '@/lib/redaction';

describe('redaction', () => {
  it('redacts bearer tokens', () => {
    const s = 'GET failed: Bearer eyJabc.def.ghi -> 500';
    const r = redactAccessTokens(s);
    expect(r).toContain('Bearer [REDACTED]');
    expect(r).not.toContain('eyJabc');
  });

  it('redacts jwt-shaped strings', () => {
    const s = 'token eyJabc.def.ghi was used';
    expect(redactPII(s)).toContain('[JWT_REDACTED]');
  });

  it('redacts emails', () => {
    const s = 'contact me at user@example.com';
    expect(redactPII(s)).toContain('[EMAIL_REDACTED]');
  });
});
