/**
 * mockDate — freezes `Date.now()` for deterministic tests.
 */

export function mockDate(iso: string): () => void {
  const original = Date.now;
  const fixed = new Date(iso).getTime();
  Date.now = () => fixed;
  return () => {
    Date.now = original;
  };
}
