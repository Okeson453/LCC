'use client';

import { useEffect } from 'react';
import { ErrorState, Button } from '@lcc/ui';

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}): React.ReactElement {
  useEffect(() => {
    // Surface to telemetry (Sentry). Avoid console.log per security policy.
    if (typeof window !== 'undefined' && (window as unknown as { Sentry?: unknown }).Sentry) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).Sentry?.captureException?.(error);
    }
  }, [error]);

  return (
    <div className="flex min-h-screen items-center justify-center p-8">
      <ErrorState
        title="Something went wrong"
        description="An unexpected error occurred. Try again, or report the issue with the trace ID below."
        traceId={error.digest}
        onRetry={reset}
      >
        <Button asChild variant="outline" size="sm">
          <a href="/today">Go to Today</a>
        </Button>
      </ErrorState>
    </div>
  );
}
