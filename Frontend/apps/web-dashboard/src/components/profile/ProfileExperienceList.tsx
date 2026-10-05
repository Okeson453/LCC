'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { ProfileExperienceItem } from '@/lib/api/profile';

export function ProfileExperienceList({ experiences }: { experiences: ProfileExperienceItem[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Experience</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {experiences.map((e, i) => (
          <div key={i} className="rounded-md border p-3">
            <p className="text-sm font-medium">{e.title} — {e.company}</p>
            <p className="text-xs text-muted-foreground">
              {e.start_date ?? '—'} → {e.current ? 'present' : e.end_date ?? '—'}
            </p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
