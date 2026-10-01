'use client';

import * as React from 'react';
import { useMutation } from '@tanstack/react-query';
import { Textarea, Button, Card, CardContent, CardHeader, CardTitle, CardDescription, Alert, AlertTitle, AlertDescription, Tabs, TabsList, TabsTrigger, TabsContent } from '@lcc/ui';
import { VariantPicker } from './VariantPicker';
import { ContentQualityPanel } from './ContentQualityPanel';
import { KbQuickPicker } from './KbQuickPicker';

export interface ComposerRootProps {
  memberId: string;
  initialPrompt?: string;
  compose: (prompt: string, variants: ('authority' | 'contrarian' | 'bts')[]) => Promise<{ variant: string; body: string }[]>;
  onSubmit: (body: string) => Promise<void>;
}

export function ComposerRoot({ memberId, initialPrompt, compose, onSubmit }: ComposerRootProps): React.ReactElement {
  const [prompt, setPrompt] = React.useState(initialPrompt ?? '');
  const [body, setBody] = React.useState('');
  const composeMutation = useMutation({ mutationFn: () => compose(prompt, ['authority', 'contrarian', 'bts']) });
  const submitMutation = useMutation({ mutationFn: () => onSubmit(body) });

  const variants = composeMutation.data ?? [];

  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Idea / prompt</CardTitle>
          <CardDescription>What do you want to post about?</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} rows={3} aria-label="Prompt" />
          <Button onClick={() => composeMutation.mutate()} disabled={composeMutation.isPending || !prompt.trim()} type="button">
            {composeMutation.isPending ? 'Generating…' : 'Generate variants'}
          </Button>
        </CardContent>
      </Card>

      {variants.length > 0 ? (
        <Tabs defaultValue="pick">
          <TabsList>
            <TabsTrigger value="pick">Pick a variant</TabsTrigger>
            <TabsTrigger value="kb">Pick KB citations</TabsTrigger>
          </TabsList>
          <TabsContent value="pick">
            <VariantPicker
              variants={variants}
              selected={null}
              onSelect={(v) => setBody(v.body)}
            />
          </TabsContent>
          <TabsContent value="kb">
            <KbQuickPicker memberId={memberId} onPick={(ids) => void ids /* attach to draft later */} />
          </TabsContent>
        </Tabs>
      ) : null}

      {body ? (
        <Card>
          <CardHeader>
            <CardTitle>Draft body</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={10} aria-label="Draft body" />
            <ContentQualityPanel memberId={memberId} body={body} onProceed={onSubmit} />
            <Button onClick={() => submitMutation.mutate()} disabled={submitMutation.isPending} type="button">
              {submitMutation.isPending ? 'Submitting…' : 'Save draft'}
            </Button>
          </CardContent>
        </Card>
      ) : null}

      {composeMutation.error ? (
        <Alert variant="destructive">
          <AlertTitle>Could not generate variants</AlertTitle>
          <AlertDescription>{String(composeMutation.error)}</AlertDescription>
        </Alert>
      ) : null}
    </div>
  );
}
