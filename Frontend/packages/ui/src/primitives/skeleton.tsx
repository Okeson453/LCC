import * as React from 'react';
import { cn } from '../utils/cn';

export interface SkeletonProps extends React.HTMLAttributes<HTMLDivElement> {
  /** Render as a circle when true. */
  rounded?: boolean;
}

export const Skeleton = React.forwardRef<HTMLDivElement, SkeletonProps>(
  ({ className, rounded, ...props }, ref) => (
    <div
      ref={ref}
      aria-hidden="true"
      aria-busy="true"
      className={cn(
        'animate-pulse bg-muted',
        rounded ? 'rounded-full' : 'rounded-md',
        className,
      )}
      {...props}
    />
  ),
);
Skeleton.displayName = 'Skeleton';
