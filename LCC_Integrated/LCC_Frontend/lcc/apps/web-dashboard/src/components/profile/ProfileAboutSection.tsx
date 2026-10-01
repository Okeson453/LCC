'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';

export function ProfileAboutSection({ about }: { about: string }): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">About</CardTitle></CardHeader>
      <CardContent><p className="whitespace-pre-line text-sm text-muted-foreground">{about}</p></CardContent>
    </Card>
  );
}
