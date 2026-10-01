# @lcc/test-utils

Shared test helpers for the OKESON-LCC frontend.

## What's in here

| Module | Purpose |
|---|---|
| `render.tsx` | Custom render that wraps with all required providers (Query, Theme, Auth, Realtime). |
| `render-hook.tsx` | Same but for testing hooks in isolation. |
| `providers/` | Individual provider stubs (Query, Theme, Auth, Realtime). |
| `msw/` | MSW server + per-resource handlers + canned fixtures. |
| `playwright/` | Page objects for E2E tests. |
| `a11y/` | axe-core runner, contrast checker. |
| `utils/` | `wait-for`, `mock-date`. |
| `mocks/` | Mock logger (with redaction), mock realtime client. |

## Usage

### Component test

```tsx
import { render, screen } from '@lcc/test-utils';
import { ApprovalQueue } from '@lcc/approval-gate';

test('renders empty state', () => {
  render(<ApprovalQueue items={[]} />);
  expect(screen.getByText(/no pending approvals/i)).toBeInTheDocument();
});
```

### Hook test

```tsx
import { renderHook, waitFor } from '@lcc/test-utils';
import { useApprovalQueue } from '@lcc/approval-gate';

test('fetches approvals', async () => {
  const { result } = renderHook(() =>
    useApprovalQueue({ fetcher: vi.fn().mockResolvedValue([/* ... */]) }),
  );
  await waitFor(() => expect(result.current.isSuccess).toBe(true));
});
```

### MSW

```tsx
import { server } from '@lcc/test-utils/msw/server';
import { http, HttpResponse } from 'msw';

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('handles 401', async () => {
  server.use(
    http.get('/api/v1/members/me', () => HttpResponse.json({ error: 'unauthorized' }, { status: 401 })),
  );
  // ...
});
```
