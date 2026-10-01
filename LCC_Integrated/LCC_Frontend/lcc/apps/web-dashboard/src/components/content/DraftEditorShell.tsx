'use client';

import * as React from 'react';
import { Textarea, Card, CardContent, CardHeader, CardTitle, Button } from '@lcc/ui';

export interface DraftEditorShellProps {
  initialBody: string;
  onSave: (body: string) => Promise<void>;
  onSubmit: (body: string) => Promise<void>;
}

export function DraftEditorShell({ initialBody, onSave, onSubmit }: DraftEditorShellProps): React.ReactElement {
  const [body, setBody] = React.useState(initialBody);
  return (
    <Card>
      <CardHeader>
        <CardTitle>Body</CardTitle>
      </CardHeader>
      <CardContent className="space-y-2">
        <Textarea value={body} onChange={(e) => setBody(e.target.value)} rows={14} aria-label="Draft body" />
        <div className="flex justify-end gap-2">
          <Button variant="outline" onClick={() => void onSave(body)} type="button">Save</Button>
          <Button onClick={() => void onSubmit(body)} type="button">Submit for approval</Button>
        </div>
      </CardContent>
    </Card>
  );
}
