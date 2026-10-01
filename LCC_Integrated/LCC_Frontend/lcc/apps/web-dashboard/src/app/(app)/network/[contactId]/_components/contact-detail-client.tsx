'use client';

import * as React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, LoadingSkeleton, ErrorState, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { getContact, listContactInteractions, logInteraction, updateContact } from '@/lib/api/network';
import type { RelationshipStage } from '@lcc/api-types';

export function ContactDetailClient({ contactId }: { contactId: string }): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  if (memberQuery.isLoading) return <LoadingSkeleton rows={6} />;
  if (memberQuery.error || !memberQuery.data) return <ErrorState onRetry={() => memberQuery.refetch()} />;
  return <Detail memberId={memberQuery.data.id} contactId={contactId} />;
}

function Detail({ memberId, contactId }: { memberId: string; contactId: string }): React.ReactElement {
  const qc = useQueryClient();
  const contact = useQuery({ queryKey: ['network', 'contact', memberId, contactId] as const, queryFn: () => getContact(memberId, contactId) });
  const interactions = useQuery({ queryKey: ['network', 'interactions', memberId, contactId] as const, queryFn: () => listContactInteractions(memberId, contactId) });
  const [stage, setStage] = React.useState<RelationshipStage | null>(null);

  React.useEffect(() => {
    if (contact.data) setStage(contact.data.stage);
  }, [contact.data]);

  const updateMutation = useMutation({
    mutationFn: () => updateContact(memberId, contactId, { stage: stage ?? undefined }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['network'] }),
  });

  const logMutation = useMutation({
    mutationFn: () => logInteraction(memberId, contactId, { kind: 'meeting', occurred_at: new Date().toISOString(), direction: 'outbound' }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['network'] }),
  });

  if (contact.isLoading) return <LoadingSkeleton rows={6} />;
  if (!contact.data) return <ErrorState title="Contact not found" />;
  const c = contact.data;

  return (
    <ComplianceGate>
      <div className="space-y-6 p-6">
        <header className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">{c.display_name}</h1>
            <p className="text-sm text-muted-foreground">{c.headline}</p>
          </div>
          <Button asChild variant="outline" size="sm">
            <Link href="/network">← Back</Link>
          </Button>
        </header>

        <Card>
          <CardHeader><CardTitle className="text-base">Stage</CardTitle></CardHeader>
          <CardContent className="flex gap-2">
            <Select value={stage ?? c.stage} onValueChange={(v) => setStage(v as RelationshipStage)}>
              <SelectTrigger className="w-48"><SelectValue /></SelectTrigger>
              <SelectContent>
                {(['cold', 'connected', 'engaged', 'conversation', 'opportunity', 'closed'] as const).map((s) => (
                  <SelectItem key={s} value={s} className="capitalize">{s}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button onClick={() => updateMutation.mutate()} disabled={updateMutation.isPending} type="button">Save</Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <CardTitle className="text-base">Timeline</CardTitle>
              <Button onClick={() => logMutation.mutate()} size="sm" disabled={logMutation.isPending} type="button">Log interaction</Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            {interactions.isLoading ? <LoadingSkeleton rows={3} /> :
              !interactions.data || interactions.data.length === 0 ? <p className="text-sm text-muted-foreground">No interactions yet.</p> :
              interactions.data.map((i) => (
                <div key={i.id} className="flex items-center justify-between rounded-md border p-2">
                  <div>
                    <p className="text-sm">{i.body || i.kind}</p>
                    <p className="text-xs text-muted-foreground">{new Date(i.occurred_at).toLocaleString()}</p>
                  </div>
                  <Badge variant="outline" className="capitalize">{i.kind}</Badge>
                </div>
              ))
            }
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
