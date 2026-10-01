import * as React from 'react';
import { cn } from '@lcc/ui';

export interface PageContainerProps {
  children: React.ReactNode;
  className?: string;
  id?: string;
}

export function PageContainer({ children, className, id }: PageContainerProps): React.ReactElement {
  return (
    <div id={id} className={cn('mx-auto w-full max-w-7xl p-6', className)}>
      {children}
    </div>
  );
}
