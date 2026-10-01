/**
 * GuardFailurePanel — denial-reason rendering with retry CTA.
 *
 * Per Frontend Design Concept §21.4: if the Compliance Governor denies, the
 * dialog shows the failed guard + reason and offers "Edit and retry" or
 * "Cancel". Never silently retries or hides a denial.
 */

import * as React from 'react';
import { Button } from '@lcc/ui';

export interface GuardFailurePanelProps {
  guardName: string;
  reason: string;
  traceId: string;
  onRetry?: () => void;
  onCancel?: () => void;
  className?: string;
  id?: string;
}

export function GuardFailurePanel({
  guardName,
  reason,
  traceId,
  onRetry,
  onCancel,
  className,
  id,
}: GuardFailurePanelProps): React.ReactElement {
  return (
    <section
      id={id}
      role="alert"
      aria-labelledby={`${id ?? 'gfp'}-heading`}
      className={[
        'rounded-md border border-destructive/40 bg-destructive/5 p-4',
        className,
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <header className="mb-3 flex items-center justify-between">
        <h4
          id={`${id ?? 'gfp'}-heading`}
          className="text-sm font-semibold text-destructive"
        >
          Action denied
        </h4>
        <span className="text-xs text-muted-foreground">trace: {traceId}</span>
      </header>
      <dl className="space-y-2 text-sm">
        <div>
          <dt className="text-xs uppercase tracking-wide text-muted-foreground">
            Failed guard
          </dt>
          <dd className="font-mono text-xs">{guardName}</dd>
        </div>
        <div>
          <dt className="text-xs uppercase tracking-wide text-muted-foreground">
            Reason
          </dt>
          <dd className="text-sm">{reason}</dd>
        </div>
      </dl>
      <footer className="mt-4 flex items-center justify-end gap-2">
        {onCancel ? (
          <Button variant="ghost" onClick={onCancel} type="button">
            Cancel
          </Button>
        ) : null}
        {onRetry ? (
          <Button variant="default" onClick={onRetry} type="button">
            Edit and retry
          </Button>
        ) : null}
      </footer>
    </section>
  );
}
