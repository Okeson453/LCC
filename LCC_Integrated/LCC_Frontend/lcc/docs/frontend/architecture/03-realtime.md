# Realtime strategy

The frontend MUST use the backend gateway as its sole connection surface — no direct
LinkedIn connections, no direct database, no third-party socket services.

## Channels

| Channel                       | Direction | Purpose                              | Forwarded via |
|-------------------------------|-----------|--------------------------------------|---------------|
| `briefing:{memberId}`         | Push      | Today cards, refresh notifications   | WS / SSE       |
| `approvals:{memberId}`        | Push      | New approvals + decisions            | WS / SSE       |
| `engagement:{memberId}`       | Push      | Inbox events, queue updates          | WS / SSE       |
| `sequence:{memberId}:{id}`    | Push      | Per-sequence step state              | WS             |
| `compliance`                  | Broadcast | New version published                | WS             |
| `integration:linkedin`        | Broadcast | Provider status                      | WS             |

## Connection selection

1. WebSocket: default, gated by `RFC: lcc.protoUp`.
2. SSE: fallback for restrictive networks.
3. Polling: 30s polling of the `/events/stream` endpoint as the lowest tier.

## Backoff + jitter

`WsConnection` uses exponential backoff capped at 30s with 250ms jitter. The
client resubscribes to its scope channels on each connect.

## Hooks

- `useChannel(scope)` returns `{ status, lastEvent }`.
- `useRealtimeInvalidator(scope, queryKeys)` invalidates TanStack Query keys on event.
