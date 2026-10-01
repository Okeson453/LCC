'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';

export interface Digest {
  id: string;
  range_label: string;
  published_at: string;
  highlights: { metric: string; value: string; direction: 'up' | 'down' | 'flat' }[];
}

export function AnalyticsDigestCard({ digest }: { digest: Digest }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{digest.range_label}</CardTitle>
          <Badge variant="outline">{new Date(digest.published_at).toLocaleDateString()}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {digest.highlights.map((h, i) => (
          <div key={i} className="flex items-center justify-between text-sm">
            <span>{h.metric}</span>
            <span>
              {h.value} {h.direction === 'up' ? '↑' : h.direction === 'down' ? '↓' : '→'}
            </span>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
