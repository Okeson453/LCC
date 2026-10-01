/**
 * Nav route definitions.
 */

export interface NavItem {
  href: string;
  label: string;
  icon?: string;
  /** Tier-badge count (e.g. pending approvals). */
  countKey?: 'pendingApprovals' | 'unreadEngagement';
  /** Section in the sidebar. */
  section: 'today' | 'pipeline' | 'studio' | 'insights' | 'settings' | 'admin';
  /** Show only for admin role. */
  adminOnly?: boolean;
}

export const NAV_ITEMS: readonly NavItem[] = [
  // Today
  { href: '/today', label: 'Today', section: 'today' },
  { href: '/approvals', label: 'Approvals', section: 'today', countKey: 'pendingApprovals' },

  // Pipeline
  { href: '/content', label: 'Content', section: 'pipeline' },
  { href: '/engagement', label: 'Engagement', section: 'pipeline', countKey: 'unreadEngagement' },
  { href: '/outreach', label: 'Outreach', section: 'pipeline' },
  { href: '/network', label: 'Network', section: 'pipeline' },
  { href: '/opportunities', label: 'Opportunities', section: 'pipeline' },

  // Studio
  { href: '/content/new', label: 'Composer', section: 'studio' },
  { href: '/profile', label: 'Profile', section: 'studio' },
  { href: '/kb', label: 'Knowledge base', section: 'studio' },

  // Insights
  { href: '/analytics', label: 'Analytics', section: 'insights' },
  { href: '/analytics/account-health', label: 'Account health', section: 'insights' },

  // Settings
  { href: '/settings/profile', label: 'Profile', section: 'settings' },
  { href: '/settings/timezone', label: 'Timezone', section: 'settings' },
  { href: '/settings/goal-mode', label: 'Goal mode', section: 'settings' },
  { href: '/settings/content-pillars', label: 'Content pillars', section: 'settings' },
  { href: '/settings/icp', label: 'ICP', section: 'settings' },
  { href: '/settings/oauth', label: 'OAuth', section: 'settings' },
  { href: '/settings/kb-rag', label: 'KB / RAG', section: 'settings' },
  { href: '/settings/data-export', label: 'Data export', section: 'settings' },
  { href: '/settings/account', label: 'Account', section: 'settings' },
  { href: '/settings/notifications', label: 'Notifications', section: 'settings' },

  // Admin
  { href: '/admin/compliance', label: 'Compliance config', section: 'admin', adminOnly: true },
  { href: '/admin/compliance/restrictions', label: 'Restrictions', section: 'admin', adminOnly: true },
];

export const NAV_SECTIONS: readonly { key: NavItem['section']; label: string }[] = [
  { key: 'today', label: 'Today' },
  { key: 'pipeline', label: 'Pipeline' },
  { key: 'studio', label: 'Studio' },
  { key: 'insights', label: 'Insights' },
  { key: 'settings', label: 'Settings' },
  { key: 'admin', label: 'Admin' },
];
