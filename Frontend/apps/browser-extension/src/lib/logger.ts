import { redactPII } from './redaction';

export const logger = {
  info(msg: string, ctx?: Record<string, unknown>): void { console.info(redactPII(format(msg, ctx))); },
  warn(msg: string, ctx?: Record<string, unknown>): void { console.warn(redactPII(format(msg, ctx))); },
  error(msg: string, ctx?: Record<string, unknown>): void { console.error(redactPII(format(msg, ctx))); },
};

function format(msg: string, ctx?: Record<string, unknown>): string {
  if (!ctx) return msg;
  try { return `${msg} ${JSON.stringify(ctx)}`; } catch { return msg; }
}
