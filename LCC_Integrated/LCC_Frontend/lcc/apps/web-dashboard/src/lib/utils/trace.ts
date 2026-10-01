/**
 * Trace ID utilities — generate, extract, log.
 */

import { generateTraceId } from '@lcc/api-types/brand';

export function newTraceId(): string {
  return generateTraceId();
}

export function extractTraceId(headers: Headers | Record<string, string>): string | null {
  if (headers instanceof Headers) {
    return headers.get('x-trace-id');
  }
  return headers['x-trace-id'] ?? null;
}
