/**
 * Re-export of the service-worker entrypoint. Vite bundles this as the
 * `background` chunk; `service-worker.ts` contains the actual lifecycle.
 */
export { default } from './service-worker';
