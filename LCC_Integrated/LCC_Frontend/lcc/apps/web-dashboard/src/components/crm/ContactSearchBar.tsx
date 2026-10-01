'use client';

import * as React from 'react';
import { Search } from 'lucide-react';
import { Input } from '@lcc/ui';

export function ContactSearchBar({ value, onChange }: { value: string; onChange: (s: string) => void }): React.ReactElement {
  return (
    <div className="relative">
      <Search className="pointer-events-none absolute left-3 top-3 h-4 w-4 text-muted-foreground" aria-hidden="true" />
      <Input
        type="search"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="Search contacts…"
        className="pl-9"
        aria-label="Search contacts"
      />
    </div>
  );
}
