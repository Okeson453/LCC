/**
 * Re-exports from `@lcc/realtime` plus the WS bridges that wire
 * RealtimeClient events into TanStack Query invalidations.
 */
export * from '@lcc/realtime';
export { useWsBridges } from '@/lib/api/ws-bridges';
export { WsDevtools } from './ws-devtools';
