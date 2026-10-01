import * as React from 'react';
import { cn } from '../utils/cn';

export interface LiveRegionProps {
  /** Politeness setting. */
  level?: 'polite' | 'assertive';
  /** Visible content (rendered with sr-only by default). */
  message: string;
  className?: string;
  id?: string;
  /** Make content visible (rare). */
  visible?: boolean;
}

export function LiveRegion({
  level = 'polite',
  message,
  className,
  id,
  visible = false,
}: LiveRegionProps): React.ReactElement {
  return (
    <div
      id={id}
      role="status"
      aria-live={level}
      aria-atomic="true"
      className={cn(!visible && 'sr-only', className)}
    >
      {message}
    </div>
  );
}
