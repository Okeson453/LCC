/**
 * waitFor — wraps `@testing-library`'s waitFor with sensible defaults.
 */

import { waitFor as rtlWaitFor, type waitForOptions } from '@testing-library/react';

export async function waitFor(
  callback: () => void | Promise<void>,
  options?: waitForOptions,
): Promise<void> {
  await rtlWaitFor(callback, { timeout: 5_000, ...options });
}
