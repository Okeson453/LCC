'use client';
import { ACTION_TYPE_TO_TIER } from "@lcc/api-types/manual/risk-tier";

import * as React from 'react';
import { useRouter } from 'next/navigation';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Sparkles, Send, Shield } from 'lucide-react';
import { Button, Textarea, Card, CardContent, CardHeader, CardTitle, CardDescription, Badge, Alert, AlertTitle, AlertDescription } from '@lcc/ui';
import { ApprovalDialog, useApprovalDialog } from '@lcc/approval-gate';
import { RiskTierBadge, KbCitationsList } from 'approval-gate';
import { ComplianceGate } from '@lcc/compliance-state';
import { fetchCurrentMember } from '@/lib/api/members';
import { composeContent, createContent, runQualityCheck, submitForApproval, scheduleContent } from '@/lib/api/content';
import { useComposerStore } from '@/lib/stores';
import type { ContentVariant, QualityFlag } from '@lcc/api-types';

export function ComposerClient(): React.ReactElement {
  const memberQuery = useQuery({ queryKey: ['members', 'me'] as const, queryFn: fetchCurrentMember });
  const composer = useComposerStore();
  const router = useRouter();

  const memberId = memberQuery.data?.id ?? null;

  const composeMutation = useMutation({
    mutationFn: ({ prompt, id }: { prompt: string; id: string }) =>
      composeContent(id, { prompt, variants: ['authority', 'contrarian', 'bts'] }),
  });

  const qualityMutation = useMutation({
    mutationFn: ({ contentId, id }: { contentId: string; id: string }) => runQualityCheck(id, contentId),
  });

  const createMutation = useMutation({
    mutationFn: ({ body, id }: { body: string; id: string }) => createContent(id, { body }),
  });

  if (memberQuery.isLoading) return null;
  if (!memberId) return (
    <div className="p-6">
      <Alert variant="destructive">
        <AlertTitle>Not signed in</AlertTitle>
        <AlertDescription>
          We couldn&apos;t load your member profile. <a href="/api/auth/linkedin/start" className="underline">Sign in</a> and try again.
        </AlertDescription>
      </Alert>
    </div>
  );

  async function onGenerate() {
    if (!memberId) return;
    const variants = await composeMutation.mutateAsync({ prompt: composer.prompt, id: memberId });
    if (variants[0]) composer.setBody(variants[0].body);
  }

  async function onQualityCheck() {
    if (!composer.body.trim() || !memberId) return;
    const item = await createMutation.mutateAsync({ body: composer.body, id: memberId });
    const report = await qualityMutation.mutateAsync({ contentId: item.id, id: memberId });
    if (report.flags.length > 0) {
      alert(`Found ${report.flags.length} flag(s). kb_coverage=${(report.kb_coverage * 100).toFixed(0)}%`);
    } else {
      alert('Quality check passed.');
    }
  }

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header>
          <h1 className="flex items-center gap-2 text-2xl font-bold">
            <Sparkles className="h-5 w-5 text-primary" aria-hidden="true" />
            Composer
          </h1>
          <p className="text-sm text-muted-foreground">Generate variants from a prompt. Approve before publishing.</p>
        </header>

        <Card>
          <CardHeader>
            <CardTitle>Idea / prompt</CardTitle>
            <CardDescription>What do you want to post about?</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Textarea
              value={composer.prompt}
              onChange={(e) => composer.setPrompt(e.target.value)}
              rows={3}
              placeholder="e.g., Why we chose Rust for our new service"
              aria-label="Prompt"
            />
            <div className="flex flex-wrap gap-2">
              <Button onClick={() => void onGenerate()} disabled={composeMutation.isPending || !composer.prompt.trim()} type="button">
                {composeMutation.isPending ? 'Generating…' : 'Generate variants'}
              </Button>
              <Button onClick={() => void onQualityCheck()} variant="outline" disabled={!composer.body.trim()} type="button">
                Quality check
              </Button>
            </div>
          </CardContent>
        </Card>

        {composeMutation.data && composeMutation.data.length > 0 ? (
          <VariantPicker
            variants={composeMutation.data}
            selected={composer.selectedVariant}
            onSelect={(v) => {
              composer.setSelectedVariant(v.variant);
              composer.setBody(v.body);
            }}
          />
        ) : null}

        {composer.body ? (
          <Card>
            <CardHeader>
              <CardTitle>Draft body</CardTitle>
            </CardHeader>
            <CardContent>
              <Textarea
                value={composer.body}
                onChange={(e) => composer.setBody(e.target.value)}
                rows={10}
                aria-label="Draft body"
              />
              <div className="mt-3 flex gap-2">
                {memberId ? <SubmitForApprovalButton memberId={memberId} body={composer.body} onSubmitted={(id) => router.push(`/content/${id}`)} /> : null}
              </div>
            </CardContent>
          </Card>
        ) : null}
      </div>
    </ComplianceGate>
  );
}

function VariantPicker({
  variants,
  selected,
  onSelect,
}: {
  variants: ContentVariant[];
  selected: string | null;
  onSelect: (v: ContentVariant) => void;
}): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Variants</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        {variants.map((v) => (
          <button
            key={v.variant}
            type="button"
            onClick={() => onSelect(v)}
            className={`w-full rounded-md border p-3 text-left ${selected === v.variant ? 'border-primary bg-primary/5' : 'hover:bg-muted/40'}`}
            data-testid="variant-preview"
          >
            <div className="flex items-center gap-2">
              <Badge variant="outline">{v.variant}</Badge>
              <RiskTierBadge tier={ACTION_TYPE_TO_TIER.publish_post} />
            </div>
            <p className="mt-2 line-clamp-3 text-sm">{v.body}</p>
          </button>
        ))}
      </CardContent>
    </Card>
  );
}

function SubmitForApprovalButton({ memberId, body, onSubmitted }: { memberId: string; body: string; onSubmitted: (id: string) => void }): React.ReactElement {
  const dialog = useApprovalDialog();
  const qc = useQueryClient();
  const [createdId, setCreatedId] = React.useState<string | null>(null);
  const submit = useMutation({ mutationFn: async () => {
    const item = await createContent(memberId, { body });
    setCreatedId(item.id);
    return submitForApproval(memberId, item.id);
  } });

  return (
    <>
      <Button onClick={() => submit.mutate()} disabled={submit.isPending} type="button">
        <Shield className="mr-1 h-4 w-4" aria-hidden="true" />
        Submit for approval
      </Button>
      {createdId && submit.data ? (
        <ApprovalDialog
          open={true}
          onOpenChange={() => onSubmitted(createdId)}
          approvalId={createdId}
          actionType="publish_post"
          tier={ACTION_TYPE_TO_TIER.publish_post}
          preview={body}
          targetLabel="LinkedIn post"
          kbRefs={[]}
          traceId={submit.data.trace_id}
          idempotencyKey={submit.data.idempotency_key ?? ''}
          onApprove={() => { qc.invalidateQueries({ queryKey: ['content'] }); console.info('[composer] content approved:', createdId); }}
          onReject={() => { console.warn('[composer] approval declined:', createdId); }}
        />
      ) : null}
    </>
  );
}

