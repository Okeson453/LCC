'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import type { ProfileExperience } from '@lcc/api-types';

export function ProfileExperienceList({ experiences }: { experiences: ProfileExperience[] }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Experience</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {experiences.map((e, i) => (
          <div key={i} className="rounded-md border p-3">
            <p className="text-sm font-medium">{e.title} — {e.company}</p>
            <p className="text-xs text-muted-foreground">
              {e.starts_at} → {e.ends_at ?? 'present'} · {e.location}
            </p>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
