import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, Textarea } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'Content pillars' };

export default function ContentPillarsPage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Content pillars</h1>
          <p className="text-sm text-muted-foreground">3–5 themes your content will orbit.</p>
        </header>
        <Card>
          <CardHeader><CardTitle>Themes</CardTitle></CardHeader>
          <CardContent>
            <Textarea rows={5} placeholder="e.g., Engineering leadership, decision-making under uncertainty, scaling teams" aria-label="Content pillars" />
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
