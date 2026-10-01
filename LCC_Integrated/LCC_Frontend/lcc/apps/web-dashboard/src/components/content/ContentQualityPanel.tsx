'use client';

import * as React from 'react';
import { useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, Button, Badge, Alert, AlertTitle, AlertDescription } from '@lcc/ui';
import { runQualityCheck } from '@/lib/api/content';
import type { QualityFlag } from '@lcc/api-types';

export interface ContentQualityPanelProps {
  memberId: string;
  body: string;
  onProceed?: (input: { score: number; flags: QualityFlag[]; kb_coverage: number }) => void;
}

export function ContentQualityPanel({ memberId, body, onProceed }: ContentQualityPanelProps): React.ReactElement {
  const qc = useMutation({
    mutationFn: async () => runQualityCheck(memberId, 'ad-hoc'),
    onSuccess: (r) => onProceed?.(r),
  });

  return (
    <Card>
      <CardHeader><CardTitle className="text-sm">Quality</CardTitle></CardHeader>
      <CardContent className="space-y-2">
        <Button onClick={() => qc.mutate()} disabled={qc.isPending || !body.trim()} variant="outline" type="button">
          {qc.isPending ? 'Checking…' : 'Run quality check'}
        </Button>

        {qc.data ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-xs">
              <Badge variant="secondary">Score {qc.data.score}</Badge>
              <Badge variant="outline">KB coverage {(qc.data.kb_coverage * 100).toFixed(0)}%</Badge>
            </div>
            {qc.data.flags.length > 0 ? (
              <Alert variant="warning">
                <AlertTitle>{qc.data.flags.length} flag(s)</AlertTitle>
                <AlertDescription>
                  <ul className="list-disc pl-4">
                    {qc.data.flags.map((f, i) => (
                      <li key={i}>{f.kind}: {f.message}</li>
                    ))}
                  </ul>
                </AlertDescription>
              </Alert>
            ) : null}
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
