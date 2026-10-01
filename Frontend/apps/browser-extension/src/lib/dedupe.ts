/**
 * Request deduplication for the extension proxy layer.
 * Audit ref: P-09.
 */
const inflight = new Map<string, Promise<unknown>>();

export function dedupe<T>(key: string, fn: () => Promise<T>): Promise<T> {
  const existing = inflight.get(key);
  if (existing) return existing as Promise<T>;
  const p = fn().finally(() => inflight.delete(key));
  inflight.set(key, p);
  return p;
}

export function buildKey(method: string, path: string): string {
  return `${method.toUpperCase()} ${path}`;
}
