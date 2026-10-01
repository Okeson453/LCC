'use client';

import * as React from 'react';
import { Textarea, Card, CardContent, CardHeader, CardTitle, Button } from '@lcc/ui';
import { ReplyVariantCard } from './ReplyVariantCard';

export interface ReplyComposerProps {
  variants: { tone: string; body: string }[];
  onApprove: (variantIndex: number, body: string) => Promise<void>;
}

export function ReplyComposer({ variants, onApprove }: ReplyComposerProps): React.ReactElement {
  const [activeIndex, setActiveIndex] = React.useState(0);
  const [body, setBody] = React.useState(variants[0]?.body ?? '');

  function pick(i: number): void {
    setActiveIndex(i);
    const v = variants[i];
    if (v) setBody(v.body);
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Compose reply</CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="grid gap-2 sm:grid-cols-3">
          {variants.map((v, i) => (
            <ReplyVariantCard key={i} tone={v.tone} body={v.body} active={activeIndex === i} onClick={() => pick(i)} />
          ))}
        </div>
        <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} aria-label="Reply body" />
        <Button onClick={() => void onApprove(activeIndex, body)} disabled={!body.trim()} type="button">
          Approve & send
        </Button>
      </CardContent>
    </Card>
  );
}
