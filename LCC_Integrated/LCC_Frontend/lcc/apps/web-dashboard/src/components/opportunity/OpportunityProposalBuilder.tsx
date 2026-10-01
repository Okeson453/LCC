'use client';
import { ACTION_TYPE_TO_TIER } from "@lcc/api-types/manual/risk-tier";

import * as React from 'react';
import { Textarea, Card, CardContent, CardHeader, CardTitle, Button } from '@lcc/ui';
import { ApprovalDialog } from '@lcc/approval-gate';

export interface OpportunityProposalBuilderProps {
  initial?: string;
  onDraft: () => Promise<string>;
  onSubmit: (body: string) => Promise<void>;
}

export function OpportunityProposalBuilder({ initial, onDraft, onSubmit }: OpportunityProposalBuilderProps): React.ReactElement {
  const [body, setBody] = React.useState(initial ?? '');
  const [open, setOpen] = React.useState(false);

  return (
    <Card>
      <CardHeader><CardTitle>Proposal</CardTitle></CardHeader>
      <CardContent className="space-y-3">
        <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={14} aria-label="Proposal body" />
        <div className="flex gap-2">
          <Button onClick={async () => setBody(await onDraft())} variant="outline" type="button">Draft with AI</Button>
          <Button onClick={async () => { await onSubmit(body); setOpen(true); }} disabled={!body.trim()} type="button">Submit for approval</Button>
        </div>
        <ApprovalDialog
          open={open}
          onOpenChange={setOpen}
          approvalId="proposal"
          actionType="send_proposal"
          tier={ACTION_TYPE_TO_TIER.send_proposal}
          preview={body}
          targetLabel="Hiring manager"
          kbRefs={[]}
          traceId=""
          idempotencyKey=""
          onApprove={async () => setOpen(false)}
          onReject={async () => setOpen(false)}
        />
      </CardContent>
    </Card>
  );
}

