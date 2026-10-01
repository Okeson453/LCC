'use client';

import * as React from 'react';
import Link from 'next/link';
import { Badge } from '@lcc/ui';
import { ContactWarmthBadge } from './ContactWarmthBadge';
import type { RelationshipStage } from '@lcc/api-types';

export interface ContactRowProps {
  id: string;
  display_name: string;
  headline: string;
  stage: RelationshipStage;
  warmth: number;
}

export function ContactRow({ id, display_name, headline, stage, warmth }: ContactRowProps): React.ReactElement {
  return (
    <Link href={`/network/${id}`} className="flex items-center justify-between rounded-md border p-3 hover:bg-muted/40">
      <div>
        <p className="text-sm font-medium">{display_name}</p>
        <p className="text-xs text-muted-foreground">{headline}</p>
      </div>
      <div className="flex items-center gap-2">
        <Badge variant="outline" className="capitalize">{stage}</Badge>
        <ContactWarmthBadge warmth={warmth} />
      </div>
    </Link>
  );
}
