'use client';

import * as React from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, Button, Input, Select, SelectTrigger, SelectContent, SelectItem, SelectValue } from '@lcc/ui';

export function LogInteractionDialog({ onSubmit }: { onSubmit: (input: { kind: string; occurred_at: string; body: string; direction: 'inbound' | 'outbound' }) => Promise<void> }): React.ReactElement {
  const [open, setOpen] = React.useState(false);
  const [kind, setKind] = React.useState('meeting');
  const [dir, setDir] = React.useState<'inbound' | 'outbound'>('outbound');
  const [body, setBody] = React.useState('');

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" type="button">Log interaction</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader><DialogTitle>Log interaction</DialogTitle></DialogHeader>
        <div className="space-y-2">
          <Select value={kind} onValueChange={setKind}><SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>{['meeting', 'message', 'call', 'email'].map((k) => <SelectItem key={k} value={k}>{k}</SelectItem>)}</SelectContent>
          </Select>
          <Select value={dir} onValueChange={(v) => setDir(v as typeof dir)}><SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent><SelectItem value="outbound">outbound</SelectItem><SelectItem value="inbound">inbound</SelectItem></SelectContent>
          </Select>
          <Input value={body} onChange={(e) => setBody(e.target.value)} placeholder="Note (optional)" aria-label="Note" />
        </div>
        <div className="mt-3 flex justify-end gap-2">
          <Button variant="outline" onClick={() => setOpen(false)} type="button">Cancel</Button>
          <Button onClick={async () => { await onSubmit({ kind, direction: dir, body, occurred_at: new Date().toISOString() }); setOpen(false); }} type="button">Save</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
