/**
 * GovernanceTrace — shows the 8-guard evaluation result.
 *
 * Renders after a decision has been recorded. If the governor permitted the
 * action, shows a `permit_token` for traceability. If denied, the
 * `GuardFailurePanel` is rendered instead.
 */

import * as React from 'react';
import type { PermitToken } from '@lcc/api-types';

export interface GovernanceEvaluation {
  permit: boolean;
  failedGuard: string | null;
  reason: string | null;
  /** Available when permit=true. */
  permitToken?: PermitToken | null;
  /** Available when permit=true. */
  expiresAt?: string | null;
}

export interface GovernanceTraceProps {
  evaluation: GovernanceEvaluation;
  traceId: string;
  className?: string;
  id?: string;
}

export function GovernanceTrace({
  evaluation,
  traceId,
  className,
  id,
}: GovernanceTraceProps): React.ReactElement {
  if (evaluation.permit) {
    return (
      <section
        id={id}
        role="status"
        aria-live="polite"
        className={[
          'rounded-md border border-success/40 bg-success/10 p-3 text-sm',
          className,
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <header className="flex items-center justify-between">
          <span className="font-medium text-success">Permitted by Compliance Governor</span>
          <span className="text-xs text-muted-foreground">trace: {traceId}</span>
        </header>
        {evaluation.permitToken ? (
          <dl className="mt-2 grid grid-cols-2 gap-1 text-xs">
            <dt className="text-muted-foreground">permit_token</dt>
            <dd className="font-mono">{evaluation.permitToken}</dd>
            {evaluation.expiresAt ? (
              <>
                <dt className="text-muted-foreground">expires_at</dt>
                <dd className="font-mono">{evaluation.expiresAt}</dd>
              </>
            ) : null}
          </dl>
        ) : null}
      </section>
    );
  }

  return (
    <section
      id={id}
      role="alert"
      aria-live="assertive"
      className={[
        'rounded-md border border-destructive/40 bg-destructive/10 p-3 text-sm',
        className,
      ]
        .filter(Boolean)
        .join(' ')}
    >
      <header className="flex items-center justify-between">
        <span className="font-medium text-destructive">Denied by Compliance Governor</span>
        <span className="text-xs text-muted-foreground">trace: {traceId}</span>
      </header>
      <dl className="mt-2 space-y-1 text-xs">
        {evaluation.failedGuard ? (
          <div className="flex gap-2">
            <dt className="text-muted-foreground">Failed guard:</dt>
            <dd className="font-mono">{evaluation.failedGuard}</dd>
          </div>
        ) : null}
        {evaluation.reason ? (
          <div className="flex gap-2">
            <dt className="text-muted-foreground">Reason:</dt>
            <dd>{evaluation.reason}</dd>
          </div>
        ) : null}
      </dl>
    </section>
  );
}
