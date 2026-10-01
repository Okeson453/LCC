import * as React from 'react';
import { cn } from '../utils/cn';
import { Button } from '../primitives/button';

export interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: React.ReactNode;
  action?: { label: string; onClick: () => void };
  className?: string;
  id?: string;
}

export function EmptyState({
  title,
  description,
  icon,
  action,
  className,
  id,
}: EmptyStateProps): React.ReactElement {
  return (
    <div
      id={id}
      role="status"
      aria-live="polite"
      className={cn(
        'flex flex-col items-center justify-center gap-3 rounded-md border border-dashed p-8 text-center',
        className,
      )}
    >
      {icon ? <div aria-hidden="true">{icon}</div> : null}
      <h3 className="text-base font-medium">{title}</h3>
      {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}
      {action ? (
        <Button variant="outline" size="sm" onClick={action.onClick} type="button">
          {action.label}
        </Button>
      ) : null}
    </div>
  );
}
