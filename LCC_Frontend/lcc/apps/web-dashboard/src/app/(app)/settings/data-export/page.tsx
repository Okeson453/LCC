'use client';

import { Metadata } from 'next';
import { useMutation } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription, Button } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'Data export' };

export default function DataExportPage(): React.ReactElement {
  return <Export />;
}

function Export(): React.ReactElement {
  const exportRequest = useMutation({ mutationFn: async () => ({ ok: true }) });
  const exportDelete = useMutation({ mutationFn: async () => ({ ok: true }) });

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Data export</h1>
          <p className="text-sm text-muted-foreground">GDPR Article 15 (access) and Article 17 (erasure) requests.</p>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Subject access request</CardTitle>
            <CardDescription>We email an export of all your data as JSON within 30 days.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => exportRequest.mutate()} disabled={exportRequest.isPending} type="button">
              {exportRequest.isPending ? 'Requesting…' : 'Request export'}
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Delete account</CardTitle>
            <CardDescription>Hard delete. Cannot be undone.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button onClick={() => exportDelete.mutate()} disabled variant="destructive" type="button">
              Delete (requires confirm via support)
            </Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
