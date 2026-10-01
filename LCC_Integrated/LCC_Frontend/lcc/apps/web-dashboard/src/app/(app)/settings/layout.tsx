import * as React from 'react';
import Link from 'next/link';
import { listSettingsNav } from './_components/settings-nav';

export default function SettingsLayout({ children }: { children: React.ReactNode }): React.ReactElement {
  const items = listSettingsNav();
  return (
    <div className="grid min-h-[calc(100vh-4rem)] gap-6 lg:grid-cols-[14rem_1fr]">
      <aside className="border-r p-4">
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">Settings</h2>
        <ul className="space-y-1 text-sm">
          {items.map((item) => (
            <li key={item.href}>
              <Link href={item.href} className="block rounded-md px-2 py-1 hover:bg-muted">
                {item.label}
              </Link>
            </li>
          ))}
        </ul>
      </aside>
      <main>{children}</main>
    </div>
  );
}
