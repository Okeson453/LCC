'use client';

/**
 * ComplianceGate — wraps an action surface. When the account is restricted,
 * children are visually disabled and become non-interactive.
 *
 * This is a defense-in-depth wrapper; the banner is the primary signal.
 * The gate provides the structural enforcement.
 */

import * as React from 'react';
import { cn } from '@lcc/ui';
import { useRestrictedState } from '../hooks/use-restricted-state';

export interface ComplianceGateProps {
  children: React.ReactNode;
  /** Render a fallback instead of children when restricted. */
  fallback?: React.ReactNode;
  /** When true, only show children when NOT restricted (inverts). */
  invert?: boolean;
  className?: string;
  id?: string;
}

export function ComplianceGate({
  children,
  fallback = null,
  invert = false,
  className,
  id,
}: ComplianceGateProps): React.ReactElement {
  const { isRestricted } = useRestrictedState();

  const hidden = invert ? !isRestricted : isRestricted;

  if (hidden) {
    return (
      <div
        id={id}
        aria-disabled="true"
        className={cn('pointer-events-none opacity-50', className)}
      >
        {fallback}
      </div>
    );
  }

  return (
    <div id={id} className={className}>
      {children}
    </div>
  );
}
