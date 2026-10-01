'use client';

import * as React from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';
import type { RelationshipStage } from '@lcc/api-types';

const STAGES: RelationshipStage[] = ['cold', 'connected', 'engaged', 'conversation', 'opportunity', 'closed'];

export function ContactStagePicker({ value, onChange }: { value: RelationshipStage; onChange: (s: RelationshipStage) => void | Promise<void> }): React.ReactElement {
  return (
    <Select value={value} onValueChange={(v) => onChange(v as RelationshipStage)}>
      <SelectTrigger className="w-48"><SelectValue /></SelectTrigger>
      <SelectContent>
        {STAGES.map((s) => <SelectItem key={s} value={s} className="capitalize">{s}</SelectItem>)}
      </SelectContent>
    </Select>
  );
}
