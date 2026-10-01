import * as React from 'react';
import { Skeleton } from '../primitives/skeleton';
import { cn } from '../utils/cn';

export interface LoadingSkeletonProps {
  /** Number of skeleton rows. */
  rows?: number;
  /** Skeleton height (e.g. "h-12"). */
  height?: string;
  /** Skeleton width class (e.g. "w-3/4"). */
  width?: string;
  className?: string;
  id?: string;
  /** Aria-label for the loading region. */
  label?: string;
}

export function LoadingSkeleton({
  rows = 3,
  height = 'h-12',
  width = 'w-full',
  className,
  id,
  label = 'Loading content',
}: LoadingSkeletonProps): React.ReactElement {
  return (
    <div
      id={id}
      role="status"
      aria-live="polite"
      aria-busy="true"
      aria-label={label}
      className={cn('space-y-2', className)}
    >
      <span className="sr-only">{label}</span>
      {Array.from({ length: rows }).map((_, i) => (
        <Skeleton key={i} className={cn(height, width)} />
      ))}
    </div>
  );
}
