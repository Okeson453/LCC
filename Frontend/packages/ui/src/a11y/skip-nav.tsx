/**
 * SkipNav — accessible skip-to-main-content link. Renders the first focusable
 * element when the user presses Tab.
 */

import * as React from 'react';
import { cn } from '../utils/cn';

export interface SkipNavProps {
  /** Target id for the main content. Defaults to "main". */
  targetId?: string;
  className?: string;
}

export function SkipNav({ targetId = 'main', className }: SkipNavProps): React.ReactElement {
  return (
    <a
      href={`#${targetId}`}
      className={cn(
        'sr-only focus:not-sr-only focus:absolute focus:left-2 focus:top-2 focus:z-50 focus:rounded-md focus:bg-background focus:px-3 focus:py-2 focus:text-sm focus:shadow',
        className,
      )}
    >
      Skip to main content
    </a>
  );
}
