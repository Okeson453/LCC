'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Dialog, DialogContent, DialogTrigger, DialogHeader, DialogTitle, DialogFooter, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';
import type { Interaction } from '@lcc/api-types';

export function ContactTimeline({ interactions, onLog }: { interactions: Interaction[]; onLog: (input: { kind: string; occurred_at: string; direction: 'inbound' | 'outbound' }) => Promise<void> }): React.ReactElement {
  const [open, setOpen] = React.useState(false);
  const [kind, setKind] = React.useState('meeting');
  const [dir, setDir] = React.useState<'inbound' | 'outbound'>('outbound');

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Timeline</CardTitle>
          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
              <Button size="sm" type="button">Log interaction</Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader><DialogTitle>Log interaction</DialogTitle></DialogHeader>
              <div className="space-y-2">
                <Select value={kind} onValueChange={setKind}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    {['meeting', 'message', 'call', 'email', 'event'].map((k) => <SelectItem key={k} value={k}>{k}</SelectItem>)}
                  </SelectContent>
                </Select>
                <Select value={dir} onValueChange={(v) => setDir(v as typeof dir)}>
                  <SelectTrigger><SelectValue /></SelectTrigger>
                  <SelectContent>
                    <SelectItem value="outbound">outbound</SelectItem>
                    <SelectItem value="inbound">inbound</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <DialogFooter>
                <Button onClick={async () => { await onLog({ kind, direction: dir, occurred_at: new Date().toISOString() }); setOpen(false); }} type="button">Save</Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {interactions.length === 0 ? (
          <p className="text-sm text-muted-foreground">No interactions yet.</p>
        ) : interactions.map((i) => (
          <div key={i.id} className="flex items-center justify-between rounded-md border p-2">
            <div>
              <p className="text-sm">{i.body || i.kind}</p>
              <p className="text-xs text-muted-foreground">{new Date(i.occurred_at).toLocaleString()}</p>
            </div>
            <Badge variant="outline" className="capitalize">{i.kind}</Badge>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
