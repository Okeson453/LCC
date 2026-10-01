/**
 * SidebarBadge — surfaces the number of pending approvals, unread messages,
 * or stale contacts. Audit ref: M-11.
 */
import * as React from 'react';
import { Badge } from '@lcc/ui';

export interface SidebarBadgeProps {
  count: number;
  tone?: 'neutral' | 'warning' | 'danger' | 'success';
  label?: string;
}

export function SidebarBadge({ count, tone = 'neutral', label }: SidebarBadgeProps): React.ReactElement | null {
  if (count <= 0) return null;
  const variant = tone === 'danger' ? 'destructive' : tone === 'warning' ? 'warning' : tone === 'success' ? 'success' : 'secondary';
  return (
    <Badge aria-label={label ?? `${count} pending`} variant={variant}>
      {count}
    </Badge>
  );
}
