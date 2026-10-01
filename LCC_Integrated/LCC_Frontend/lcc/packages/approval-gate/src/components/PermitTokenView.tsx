'use client';

/**
 * PermitTokenView — debug-only display of the permit_token.
 *
 * Hidden in production unless `?debug=permit-tokens` is in the URL. This is
 * a developer aid for inspecting governance outcomes during development and
 * incident response.
 */

import * as React from 'react';
import { cn } from '@lcc/ui';

export interface PermitTokenViewProps {
  permitToken: string;
  expiresAt?: string | null;
  traceId?: string | null;
  className?: string;
  id?: string;
}

export function PermitTokenView({
  permitToken,
  expiresAt,
  traceId,
  className,
  id,
}: PermitTokenViewProps): React.ReactElement | null {
  const [show, setShow] = React.useState(false);

  React.useEffect(() => {
    if (typeof window === 'undefined') return;
    const params = new URLSearchParams(window.location.search);
    if (params.get('debug') === 'permit-tokens') setShow(true);
  }, []);

  if (!show) return null;

  return (
    <details
      id={id}
      className={cn(
        'rounded-md border border-warning/40 bg-warning/10 p-2 text-xs',
        className,
      )}
    >
      <summary className="cursor-pointer font-medium">Permit Token (debug)</summary>
      <dl className="mt-2 space-y-1 font-mono">
        <div>
          <dt className="inline text-muted-foreground">token: </dt>
          <dd className="inline break-all">{permitToken}</dd>
        </div>
        {expiresAt ? (
          <div>
            <dt className="inline text-muted-foreground">expires: </dt>
            <dd className="inline">{expiresAt}</dd>
          </div>
        ) : null}
        {traceId ? (
          <div>
            <dt className="inline text-muted-foreground">trace: </dt>
            <dd className="inline break-all">{traceId}</dd>
          </div>
        ) : null}
      </dl>
    </details>
  );
}
