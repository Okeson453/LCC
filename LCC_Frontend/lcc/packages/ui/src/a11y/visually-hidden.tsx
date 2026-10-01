import * as React from 'react';
import * as VisuallyHiddenPrimitive from '@radix-ui/react-visually-hidden';
import { cn } from '../utils/cn';

export interface VisuallyHiddenProps extends React.HTMLAttributes<HTMLSpanElement> {}

export const VisuallyHidden = React.forwardRef<HTMLSpanElement, VisuallyHiddenProps>(
  ({ className, ...props }, ref) => (
    <VisuallyHiddenPrimitive.Root
      ref={ref}
      className={cn(className)}
      {...props}
    />
  ),
);
VisuallyHidden.displayName = 'VisuallyHidden';
