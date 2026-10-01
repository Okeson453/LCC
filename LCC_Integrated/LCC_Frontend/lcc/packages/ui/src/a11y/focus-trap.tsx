import * as React from 'react';
import { FocusTrap as RadixFocusTrap } from '@radix-ui/react-focus-scope';

export interface FocusTrapProps {
  children: React.ReactNode;
  /** When false, focus is not trapped. */
  active?: boolean;
  className?: string;
}

export function FocusTrap({ children, active = true, className }: FocusTrapProps): React.ReactElement {
  if (!active) return <div className={className}>{children}</div>;
  return (
    <RadixFocusTrap trapped loop asChild>
      <div className={className}>{children}</div>
    </RadixFocusTrap>
  );
}
