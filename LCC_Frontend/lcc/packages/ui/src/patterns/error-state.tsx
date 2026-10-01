import * as React from 'react';
import { AlertCircle } from 'lucide-react';
import { cn } from '../utils/cn';
import { Button } from '../primitives/button';

export interface ErrorStateProps {
  title?: string;
  description?: string;
  /** Retry handler — when provided, shows a "Try again" button. */
  onRetry?: () => void;
  /** Trace ID for support. */
  traceId?: string;
  className?: string;
  id?: string;
}

export function ErrorState({
  title = 'Something went wrong',
  description = 'Please try again. If the problem persists, contact support.',
  onRetry,
  traceId,
  className,
  id,
}: ErrorStateProps): React.ReactElement {
  return (
    <div
      id={id}
      role="alert"
      aria-live="assertive"
      className={cn(
        'flex flex-col items-center justify-center gap-3 rounded-md border border-destructive/40 bg-destructive/5 p-8 text-center',
        className,
      )}
    >
      <AlertCircle className="h-8 w-8 text-destructive" aria-hidden="true" />
      <h3 className="text-base font-medium text-destructive">{title}</h3>
      <p className="text-sm text-muted-foreground">{description}</p>
      {onRetry ? (
        <Button variant="outline" size="sm" onClick={onRetry} type="button">
          Try again
        </Button>
      ) : null}
      {traceId ? (
        <p className="mt-1 font-mono text-xs text-muted-foreground">
          trace: {traceId}
        </p>
      ) : null}
    </div>
  );
}
