'use client';

import * as React from 'react';
import Link from 'next/link';
import { Bell, Search, User, Sparkles } from 'lucide-react';
import { Button, Badge, ModeToggle, DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, Input } from '@lcc/ui';
import { useNotificationStore } from '@/lib/stores';
import { useClientSession } from '@/lib/auth/client-session';

export function TopBar(): React.ReactElement {
  const session = useClientSession();
  const unread = useNotificationStore((s) => s.unreadCount);

  return (
    <header className="sticky top-0 z-30 flex h-14 items-center justify-between border-b bg-background/95 px-4 backdrop-blur">
      <div className="flex items-center gap-2">
        <Link href="/today" className="flex items-center gap-2 font-semibold">
          <Sparkles className="h-5 w-5 text-primary" aria-hidden="true" />
          <span className="hidden sm:inline">LinkedIn Manager</span>
        </Link>
      </div>

      <div className="flex flex-1 items-center justify-center px-4">
        <div className="relative w-full max-w-md">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
          <Input
            type="search"
            placeholder="Search contacts, content, opportunities… (⌘K)"
            className="pl-9"
            aria-label="Global search"
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button asChild variant="ghost" size="icon" aria-label="Notifications">
          <Link href="/approvals">
            <Bell className="h-4 w-4" aria-hidden="true" />
            {unread > 0 ? (
              <Badge variant="destructive" className="absolute -right-1 -top-1 h-4 min-w-4 px-1 text-[10px]">
                {unread}
              </Badge>
            ) : null}
          </Link>
        </Button>

        <ModeToggle />

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" aria-label="Account menu">
              <User className="h-4 w-4" aria-hidden="true" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-56">
            <DropdownMenuLabel>{session.session?.user?.email ?? 'Signed in'}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem asChild>
              <Link href="/settings/profile">Profile</Link>
            </DropdownMenuItem>
            <DropdownMenuItem asChild>
              <Link href="/settings/oauth">OAuth</Link>
            </DropdownMenuItem>
            <DropdownMenuItem asChild>
              <Link href="/settings/data-export">Data export</Link>
            </DropdownMenuItem>
            <DropdownMenuItem asChild>
              <Link href="/settings/account">Delete account</Link>
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem asChild>
              <Link href="/auth/logout">Sign out</Link>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </header>
  );
}
