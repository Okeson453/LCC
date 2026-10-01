'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge, Button } from '@lcc/ui';

export function CompliancePosturePanel({ version, environment }: { version: string; environment: 'dev' | 'staging' | 'prod' }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Compliance posture</CardTitle>
          <Badge variant={environment === 'prod' ? 'success' : 'secondary'}>{environment}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-xs">Active version: <span className="font-mono">{version}</span></p>
        <Button asChild size="sm" variant="outline"><a href="/admin/compliance">Open admin</a></Button>
      </CardContent>
    </Card>
  );
}
