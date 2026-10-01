'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';
import { RiskTierBadge } from '@lcc/approval-gate';

export interface Variant {
  variant: string;
  body: string;
}

export interface VariantPickerProps {
  variants: Variant[];
  selected: string | null;
  onSelect: (v: Variant) => void;
}

export function VariantPicker({ variants, selected, onSelect }: VariantPickerProps): React.ReactElement {
  return (
    <Card>
      <CardHeader><CardTitle className="text-base">Variants</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        {variants.map((v) => (
          <button
            key={v.variant}
            type="button"
            onClick={() => onSelect(v)}
            className={`w-full rounded-md border p-3 text-left ${selected === v.variant ? 'border-primary bg-primary/5' : 'hover:bg-muted/40'}`}
          >
            <div className="flex items-center gap-2">
              <Badge variant="outline">{v.variant}</Badge>
              <RiskTierBadge tier={2} />
            </div>
            <p className="mt-2 line-clamp-3 text-sm">{v.body}</p>
          </button>
        ))}
      </CardContent>
    </Card>
  );
}
