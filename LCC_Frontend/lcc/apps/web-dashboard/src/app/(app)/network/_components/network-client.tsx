'use client';

import * as React from 'react';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Badge, LoadingSkeleton, ErrorState, Input } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listContacts } from '@/lib/api/network';

export function NetworkClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <List memberId={memberQuery.data.id} />;
}

function List({ memberId }: { memberId: string }): React.ReactElement {
  const [search, setSearch] = React.useState('');
  const q = useQuery({
    queryKey: ['network', 'contacts', memberId] as const,
    queryFn: () => listContacts(memberId, { expand: 'company' }),
  });

  const filtered = (q.data ?? []).filter((c) =>
    c.display_name.toLowerCase().includes(search.toLowerCase()) ||
    c.headline.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Network</h1>
            <p className="text-sm text-muted-foreground">Your CRM.</p>
          </div>
          <Link href="/network/stale" className="text-sm text-primary hover:underline">
            View stale contacts →
          </Link>
        </header>

        <Input
          type="search"
          placeholder="Search by name or headline…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          aria-label="Search contacts"
        />

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">Contacts</CardTitle>
              <Badge variant="secondary">{filtered.length}</Badge>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            {q.isLoading ? <LoadingSkeleton rows={4} /> :
              filtered.length === 0 ? <p className="text-sm text-muted-foreground">No contacts match.</p> :
              filtered.map((c) => (
                <Link key={c.id} href={`/network/${c.id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
                  <div>
                    <p className="text-sm font-medium">{c.display_name}</p>
                    <p className="text-xs text-muted-foreground">{c.headline}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="capitalize">{c.stage}</Badge>
                    <Badge variant="secondary">{(c.warmth_score * 100).toFixed(0)}°</Badge>
                  </div>
                </Link>
              ))
            }
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
