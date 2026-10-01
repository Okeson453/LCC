/**
 * Popup route map. Audit ref: M-31 (route naming).
 */
export type PopupRoute = '/' | '/queue' | '/kb' | '/analytics';

export function isPopupRoute(value: string): value is PopupRoute {
  return ['/', '/queue', '/kb', '/analytics'].includes(value);
}

export function defaultPopupRoute(): PopupRoute {
  return '/queue';
}
