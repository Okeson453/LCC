'use client';

/**
 * RestrictedStateBanner — global banner shown when `is_restricted=true`.
 *
 * Mounted at the root of every authenticated layout. Renders the active
 * reason with a CTA to contact support. The banner uses semantic markup
 * (role="alert", aria-live="assertive") so it is announced by screen readers
 * and immediately catches user attention.
 */

import * as React from 'react';
import { Button, cn } from '@lcc/ui';
import { useRestrictedState } from '../hooks/use-restricted-state';
import { RESTRICTION_REASON_LABEL, RESTRICTION_REASON_DESCRIPTION } from '../utils/state-derivation';

export interface RestrictedStateBannerProps {
  /** CTA target — defaults to support page. */
  onContactSupport?: () => void;
  className?: string;
  id?: string;
}

export function RestrictedStateBanner({
  onContactSupport,
  className,
  id = 'restricted-state-banner',
}: RestrictedStateBannerProps): React.ReactElement | null {
  const { isRestricted, derived } = useRestrictedState();

  if (!isRestricted || derived.kind !== 'active') return null;

  const reason = derived.reason;
  const label = RESTRICTION_REASON_LABEL[reason];
  const description = RESTRICTION_REASON_DESCRIPTION[reason];

  return (
    <div
      id={id}
      role="alert"
      aria-live="assertive"
      className={cn(
        'border-b border-destructive/40 bg-destructive/10 px-4 py-3 text-destructive-foreground',
        className,
      )}
      style={{ color: 'var(--color-restricted)' }}
    >
      <div className="mx-auto flex max-w-7xl flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-0.5">
          <p className="text-sm font-medium">
            <span aria-hidden="true">⚠ </span>
            Account restricted — {label}
          </p>
          <p className="text-xs">{description}</p>
        </div>
        {onContactSupport ? (
          <Button
            variant="outline"
            size="sm"
            onClick={onContactSupport}
            type="button"
            aria-label="Contact support to resolve restriction"
          >
            Contact support
          </Button>
        ) : null}
      </div>
    </div>
  );
}
