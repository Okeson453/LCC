'use client';

import * as React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Home, FileText, Users, MessageCircle, Send, Briefcase, User, BarChart3, Settings, ShieldCheck, Sparkles, PanelLeftClose, PanelLeftOpen, Network, Database } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { Button, cn } from '@lcc/ui';
import { useSidebarStore } from '@/lib/stores';
import { NAV_SECTIONS, NAV_ITEMS, type NavItem } from '@/types/nav';
import { listApprovals } from '@/lib/api/approval';
import { Badge } from '@lcc/ui';

const ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  '/today': Home,
  '/approvals': ShieldCheck,
  '/content': FileText,
  '/engagement': MessageCircle,
  '/outreach': Send,
  '/network': Network,
  '/opportunities': Briefcase,
  '/content/new': Sparkles,
  '/profile': User,
  '/kb': Database,
  '/analytics': BarChart3,
  '/analytics/account-health': BarChart3,
  '/settings/profile': Settings,
  '/admin/compliance': ShieldCheck,
};

export function Sidebar(): React.ReactElement {
  const pathname = usePathname();
  const collapsed = useSidebarStore((s) => s.collapsed);
  const toggle = useSidebarStore((s) => s.toggle);

  const { data: pendingApprovals } = useQuery({
    queryKey: ['approvals', 'pending-count'] as const,
    queryFn: async () => {
      const { listApprovals } = await import('@/lib/api/approval');
      return listApprovals('', { status: 'pending' }).catch(() => [] as Awaited<ReturnType<typeof listApprovals>>);
    },
    staleTime: 30_000,
  });

  return (
    <aside
      aria-label="Primary navigation"
      className={cn(
        'sticky top-14 h-[calc(100vh-3.5rem)] border-r bg-background transition-all',
        collapsed ? 'w-14' : 'w-60',
      )}
    >
      <div className="flex h-full flex-col">
        <nav className="flex-1 overflow-y-auto p-2" role="navigation">
          {NAV_SECTIONS.map((section) => {
            const items = NAV_ITEMS.filter((i) => i.section === section.key);
            return (
              <div key={section.key} className="mb-4">
                {!collapsed ? (
                  <h2 className="mb-1 px-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                    {section.label}
                  </h2>
                ) : null}
                <ul className="space-y-0.5">
                  {items.map((item) => (
                    <li key={item.href}>
                      <SidebarLink
                        item={item}
                        active={pathname === item.href || pathname.startsWith(`${item.href}/`)}
                        count={
                          item.countKey === 'pendingApprovals' && pendingApprovals
                            ? pendingApprovals.length
                            : undefined
                        }
                        collapsed={collapsed}
                      />
                    </li>
                  ))}
                </ul>
              </div>
            );
          })}
        </nav>

        <footer className="border-t p-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={toggle}
            className="w-full justify-start"
            type="button"
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
          >
            {collapsed ? <PanelLeftOpen className="h-4 w-4" aria-hidden="true" /> : <PanelLeftClose className="h-4 w-4" aria-hidden="true" />}
            {!collapsed ? 'Collapse' : null}
          </Button>
        </footer>
      </div>
    </aside>
  );
}

function SidebarLink({
  item,
  active,
  count,
  collapsed,
}: {
  item: NavItem;
  active: boolean;
  count?: number;
  collapsed: boolean;
}): React.ReactElement {
  const Icon = ICONS[item.href] ?? Home;
  return (
    <Link
      href={item.href}
      aria-current={active ? 'page' : undefined}
      className={cn(
        'flex items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors',
        active
          ? 'bg-accent text-accent-foreground'
          : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground',
      )}
      title={collapsed ? item.label : undefined}
    >
      <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
      {!collapsed ? <span className="flex-1 truncate">{item.label}</span> : null}
      {!collapsed && typeof count === 'number' && count > 0 ? (
        <Badge variant="secondary" className="ml-auto text-[10px]">
          {count}
        </Badge>
      ) : null}
    </Link>
  );
}
