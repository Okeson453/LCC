'use client';

import { Metadata } from 'next';
import Link from 'next/link';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, LoadingSkeleton, ErrorState, Badge, Input, EmptyState } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { listKbRecords } from '@/lib/api/kb';
import { Plus, Search } from 'lucide-react';
import React from 'react';

export const metadata: Metadata = { title: 'Knowledge Base' };

export default function KbPage(): React.ReactElement {
  return <Kb />;
}

function Kb(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <List memberId={memberQuery.data.id} />;
}

function List({ memberId }: { memberId: string }): React.ReactElement {
  const [search, setSearch] = React.useState('');
  const q = useQuery({ queryKey: ['kb', 'list', memberId] as const, queryFn: () => listKbRecords(memberId) });

  const filtered = (q.data ?? []).filter((r) => r.title.toLowerCase().includes(search.toLowerCase()) || r.body.toLowerCase().includes(search.toLowerCase()));

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Knowledge Base</h1>
            <p className="text-sm text-muted-foreground">Source of truth for citations.</p>
          </div>
          <Button asChild>
            <Link href="/kb/new"><Plus className="mr-1 h-4 w-4" aria-hidden="true" />New record</Link>
          </Button>
        </header>

        <div className="relative">
          <Search className="pointer-events-none absolute left-3 top-3 h-4 w-4 text-muted-foreground" aria-hidden="true" />
          <Input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search KB…"
            className="pl-9"
            aria-label="Search KB"
          />
        </div>

        {q.isLoading ? <LoadingSkeleton rows={4} /> :
          filtered.length === 0 ? <EmptyState title="No KB records" /> :
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {filtered.map((r) => (
              <Link key={r.id} href={`/kb/${r.id}`}>
                <Card>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-sm">{r.title}</CardTitle>
                      <Badge variant="outline">{r.kind}</Badge>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="line-clamp-3 text-xs text-muted-foreground">{r.body.slice(0, 160)}</p>
                  </CardContent>
                </Card>
              </Link>
            ))}
          </div>
        }
      </div>
    </ComplianceGate>
  );
}
