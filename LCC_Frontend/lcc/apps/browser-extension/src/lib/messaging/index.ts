/**
 * Typed cross-context messaging protocol. Audit ref: M-28.
 */
import type { BackgroundMessage, ContentMessage } from '../../types';

export type Message = BackgroundMessage | ContentMessage;

export const MSG = {
  openSidePanel: 'lcc.openSidePanel' as const,
  refresh: 'lcc.refresh' as const,
  fetcher: 'lcc.fetcher' as const,
  badgeUpdate: 'lcc.badgeUpdate' as const,
  compliance: 'lcc.compliance' as const,
  detectedSurface: 'lcc.detectedSurface' as const,
  actionClicked: 'lcc.actionClicked' as const,
  applyRestrictions: 'lcc.applyRestrictions' as const,
  apiProxy: 'lcc.apiProxy' as const,
  exchangeToken: 'lcc.exchangeToken' as const,
  refreshToken: 'lcc.refreshToken' as const,
};

export function isMessage<T extends Message['type']>(
  msg: unknown,
  type: T,
): msg is Extract<Message, { type: T }> {
  return Boolean(msg && typeof msg === 'object' && (msg as { type?: string }).type === type);
}

export function send<T extends Message>(msg: T): Promise<unknown> {
  return chrome.runtime.sendMessage(msg).catch(() => undefined);
}
