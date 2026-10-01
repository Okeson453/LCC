/**
 * MSW worker — browser test environment (Storybook, Vitest browser mode).
 */

import { setupWorker } from 'msw/browser';
import { handlers } from './handlers';

export const worker = setupWorker(...handlers);
