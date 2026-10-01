/**
 * Vitest setup for the Web Dashboard. Registers jest-dom matchers,
 * suppresses noisy act() warnings, and configures our logger.
 */
import '@testing-library/jest-dom/vitest';
import { afterEach, beforeAll, vi } from 'vitest';
import { cleanup } from '@testing-library/react';

beforeAll(() => {
  // MatchMedia is not implemented in jsdom but used by some hooks.
  if (!window.matchMedia) {
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  }
});

afterEach(() => {
  cleanup();
});

// Cypress / fetch stub defaults are added per-test inside the api mocks
// package under @lcc/test-utils. Default fetch to no-op so unit tests don't
// accidentally hit a real network.
if (typeof globalThis.fetch === 'undefined') {
  globalThis.fetch = vi.fn();
}
