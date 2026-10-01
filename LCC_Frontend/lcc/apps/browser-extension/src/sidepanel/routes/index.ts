/**
 * Sidepanel route map. Audit ref: M-33.
 */
export type SideRoute = '/approvals' | '/compliance' | '/settings';

export function isSideRoute(value: string): value is SideRoute {
  return ['/approvals', '/compliance', '/settings'].includes(value);
}

export function defaultSideRoute(): SideRoute {
  return '/approvals';
}
