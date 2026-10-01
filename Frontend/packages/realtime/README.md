# @lcc/realtime

Shared WebSocket + SSE + polling client for OKESON-LCC.

## Architecture

```
RealtimeClient (singleton)
  │
  ├─► WS connection (primary)
  │     ├─ reconnect with exponential backoff + jitter
  │     ├─ heartbeat ping/pong
  │     └─ auto-fallback to SSE on repeated failures
  │
  ├─► SSE connection (fallback)
  │     └─ reconnect with backoff
  │
  └─► polling (last resort)
        └─ 30s interval, exponential backoff on errors
```

## Channels

| Channel | Purpose |
|---|---|
| `ws/briefing` | Daily briefing refresh |
| `ws/approvals` | Pending-approval queue refresh |
| `ws/engagement` | Inbox / queue refresh |
| `ws/compliance` | Restriction state push |
| `ws/sequence` | Sequence timeline live updates |
| `ws/integration` | Track B (extension-only) |

## Usage

```ts
import { useBriefingChannel, useComplianceChannel } from '@lcc/realtime';

function TodayPage() {
  const { data, status, lastUpdated } = useBriefingChannel(memberId);
  // status: 'connecting' | 'connected' | 'reconnecting' | 'sse' | 'polling' | 'disconnected'
  if (status !== 'connected') {
    return <RealtimeStatusIndicator status={status} />;
  }
  // ... render
}
```

## Resilience

| Behavior | Implementation |
|---|---|
| WS drop | Auto-reconnect with exponential backoff (max 30s) + jitter |
| WS sustained failure | Fall through to SSE |
| SSE drop | Reconnect with backoff |
| All transports down | Polling at 30s |
| Visible indicator | `<RealtimeStatusIndicator>` exposed by `use-channel` |

## Non-negotiable

No business state lives in the realtime client. The client delivers events to TanStack Query cache invalidators; it does not maintain its own state stores.
