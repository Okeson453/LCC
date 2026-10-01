'use client';

import * as React from 'react';
import { useQuery } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Checkbox, Badge } from '@lcc/ui';
import { listKbRecords } from '@/lib/api/kb';

export function KbQuickPicker({ memberId, onPick, selected: initial = [] as string[] }: { memberId: string; onPick: (ids: string[]) => void; selected?: string[] }): React.ReactElement {
  const { data } = useQuery({ queryKey: ['kb', 'list', memberId] as const, queryFn: () => listKbRecords(memberId) });
  const [selected, setSelected] = React.useState<Set<string>>(new Set(initial));

  function toggle(id: string): void {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      onPick([...next]);
      return next;
    });
  }

  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Pick KB citations</CardTitle></CardHeader>
      <CardContent className="space-y-1">
        {(data ?? []).slice(0, 12).map((r) => (
          <label key={r.id} className="flex items-center justify-between rounded-md border p-2 text-xs hover:bg-muted/40">
            <span className="flex items-center gap-2">
              <Checkbox checked={selected.has(r.id)} onCheckedChange={() => toggle(r.id)} aria-label={`Select ${r.title}`} />
              {r.title}
            </span>
            <Badge variant="outline">{r.kind}</Badge>
          </label>
        ))}
      </CardContent>
    </Card>
  );
}
