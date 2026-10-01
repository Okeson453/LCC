/**
 * useRestrictedState — exposes the current restriction state.
 *
 * Reads from the `ComplianceContext`. The provider subscribes to the backend
 * (HTTP + WebSocket) and updates the context value when the state changes.
 */

import * as React from 'react';
import type { RestrictionStateDTO } from '@lcc/api-types';
import { deriveRestrictionState, type DerivedRestrictionState } from '../utils/state-derivation';

export interface UseRestrictedStateResult {
  raw: RestrictionStateDTO | null;
  derived: DerivedRestrictionState;
  isRestricted: boolean;
  refetch: () => Promise<void>;
}

export interface RestrictedStateContextValue {
  raw: RestrictionStateDTO | null;
  derived: DerivedRestrictionState;
  refetch: () => Promise<void>;
}

const Ctx = React.createContext<RestrictedStateContextValue | null>(null);

export const ComplianceContext = Ctx;

export function useRestrictedState(): UseRestrictedStateResult {
  const ctx = React.useContext(Ctx);
  if (!ctx) {
    throw new Error('useRestrictedState must be used within ComplianceProvider');
  }
  return {
    raw: ctx.raw,
    derived: ctx.derived,
    isRestricted: ctx.derived.kind === 'active',
    refetch: ctx.refetch,
  };
}
