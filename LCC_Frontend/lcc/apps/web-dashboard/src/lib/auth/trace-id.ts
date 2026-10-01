/**
 * Trace ID generation — for server components and route handlers.
 */

import { generateTraceId } from '@lcc/api-types/brand';

export function newTraceId(): string {
  return generateTraceId();
}
