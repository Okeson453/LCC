export interface SettingsNavItem {
  label: string;
  href: string;
  description?: string;
  role?: 'member' | 'admin';
}

export function listSettingsNav(): SettingsNavItem[] {
  return [
    { label: 'Profile', href: '/settings/profile', description: 'Display name, headline, avatar' },
    { label: 'Timezone', href: '/settings/timezone' },
    { label: 'Goal mode', href: '/settings/goal-mode' },
    { label: 'Content pillars', href: '/settings/content-pillars' },
    { label: 'ICP', href: '/settings/icp' },
    { label: 'OAuth', href: '/settings/oauth' },
    { label: 'KB & RAG', href: '/settings/kb-rag', description: 'Manage knowledge base ingestion' },
    { label: 'Data export', href: '/settings/data-export', description: 'Subject access requests' },
    { label: 'Account', href: '/settings/account' },
    { label: 'Notifications', href: '/settings/notifications' },
    { label: 'Compliance', href: '/admin/compliance', role: 'admin' },
  ];
}
