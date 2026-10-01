'use client';

import * as React from 'react';
import { Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';

export function TemplateFilterBar({ value, onChange }: { value: { q: string; persona: string }; onChange: (v: { q: string; persona: string }) => void }): React.ReactElement {
  return (
    <div className="flex gap-2">
      <Input placeholder="Search templates…" value={value.q} onChange={(e) => onChange({ ...value, q: e.target.value })} aria-label="Search templates" />
      <Select value={value.persona} onValueChange={(v) => onChange({ ...value, persona: v })}>
        <SelectTrigger className="w-40"><SelectValue placeholder="Persona" /></SelectTrigger>
        <SelectContent>
          <SelectItem value="all">all</SelectItem>
          <SelectItem value="recruiter">recruiter</SelectItem>
          <SelectItem value="founder">founder</SelectItem>
          <SelectItem value="pm">pm</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}
