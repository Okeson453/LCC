import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, CardDescription, Button, Textarea } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'New compliance version' };

export default function NewComplianceVersionPage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">New compliance version</h1>
          <p className="text-sm text-muted-foreground">Upload a YAML/JSON policy bundle.</p>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Bundle</CardTitle>
            <CardDescription>Validated server-side; published version becomes immutable.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Textarea rows={20} placeholder="Paste YAML or JSON here…" aria-label="Bundle" />
            <Button type="button" disabled>Validate & save (server side)</Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
