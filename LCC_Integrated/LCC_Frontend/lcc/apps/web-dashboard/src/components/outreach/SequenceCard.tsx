'use client';

import * as React from 'react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';
import { SequenceStatusBadge } from './SequenceStatusBadge';
import type { OutreachSequence } from '@lcc/api-types';

export function SequenceCard({ sequence }: { sequence: OutreachSequence }): React.ReactElement {
  return (
    <Link href={`/outreach/${sequence.id}`}>
      <Card className="hover:border-primary/40">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-sm">{sequence.name}</CardTitle>
            <SequenceStatusBadge status={sequence.status} />
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>{sequence.step_count} steps</span>
            <Badge variant="secondary">{sequence.enrolled_count} enrolled</Badge>
          </div>
        </CardContent>
      </Card>
    </Link>
  );
}
