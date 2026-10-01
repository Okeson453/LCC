'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Input, Button, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';

export function NewSequenceForm({ onCreate, templates }: { onCreate: (input: { name: string; template_id: string }) => Promise<void>; templates: { id: string; name: string }[] }): React.ReactElement {
  const [name, setName] = React.useState('');
  const [templateId, setTemplateId] = React.useState('');
  return (
    <Card>
      <CardHeader><CardTitle>New sequence</CardTitle></CardHeader>
      <CardContent className="space-y-3">
        <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="Sequence name" aria-label="Sequence name" />
        <Select value={templateId} onValueChange={setTemplateId}>
          <SelectTrigger><SelectValue placeholder="Template" /></SelectTrigger>
          <SelectContent>
            {templates.map((t) => <SelectItem key={t.id} value={t.id}>{t.name}</SelectItem>)}
          </SelectContent>
        </Select>
        <Button onClick={() => void onCreate({ name, template_id: templateId })} disabled={!name || !templateId} type="button">
          Create sequence
        </Button>
      </CardContent>
    </Card>
  );
}
