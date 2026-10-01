/**
 * Role-based access control — checks for admin, member, etc.
 *
 * Per audit S-03: real role lookup comes from the JWT's `roles` claim
 * exposed via the next-auth callback session. The Member.role surface
 * mirrors this claim. The email-based heuristic was removed.
 *
 * Used by `(app)/admin/*` layout to gate routes, and by individual pages
 * to render or hide controls.
 */

import type { Member } from '@lcc/api-types';

export type Role = 'member' | 'admin' | 'super_admin';

const ROLE_RANK: Record<Role, number> = {
  member: 1,
  admin: 2,
  super_admin: 3,
};

export function isRole(member: Member | null | undefined, role: Role): boolean {
  if (!member) return false;
  const memberRole = (member.role ?? 'member') as Role;
  return ROLE_RANK[memberRole] >= ROLE_RANK[role];
}

export function hasRole(member: Member | null | undefined, role: Role): boolean {
  return isRole(member, role);
}

export function requireRole(member: Member | null | undefined, role: Role): void {
  if (!hasRole(member, role)) {
    throw new Error(`Forbidden: required role ${role}`);
  }
}
