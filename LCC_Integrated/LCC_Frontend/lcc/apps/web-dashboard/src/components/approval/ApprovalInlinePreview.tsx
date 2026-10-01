'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';
import { RiskTierBadge } from '@lcc/approval-gate';
import type { Approval } from '@lcc/api-types';

export function ApprovalInlinePreview({ approval }: { approval: Pick<Approval, 'action_type' | 'tier' | 'payload' | 'created_at'> }): React.ReactElement {
  const preview = approval.payload && typeof approval.payload === 'object' && 'preview' in approval.payload
    ? String((approval.payload as Record<string, unknown>).preview).slice(0, 280)
    : '';
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{approval.action_type}</CardTitle>
          <RiskTierBadge tier={approval.tier} dotOnly />
        </div>
      </CardHeader>
      <CardContent>
        <p className="line-clamp-3 text-xs text-muted-foreground">{preview}</p>
        <Badge variant="outline" className="mt-2 text-xs">
          {new Date(approval.created_at).toLocaleString()}
        </Badge>
      </CardContent>
    </Card>
  );
}
