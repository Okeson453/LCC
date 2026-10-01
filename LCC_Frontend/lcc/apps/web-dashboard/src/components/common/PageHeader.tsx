import * as React from 'react';
import { cn } from '@lcc/ui';

export interface PageHeaderProps {
  title: string;
  description?: string;
  actions?: React.ReactNode;
  className?: string;
  id?: string;
}

export function PageHeader({ title, description, actions, className, id }: PageHeaderProps): React.ReactElement {
  return (
    <header id={id} className={cn('flex flex-col gap-2 border-b bg-background p-6 sm:flex-row sm:items-center sm:justify-between', className)}>
      <div>
        <h1 className="text-2xl font-bold tracking-tight">{title}</h1>
        {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}
      </div>
      {actions ? <div className="flex items-center gap-2">{actions}</div> : null}
    </header>
  );
}
