import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, CardDescription, Input } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'ICP' };

export default function IcpPage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Ideal customer profile</h1>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>ICP</CardTitle>
            <CardDescription>Titles, industries, geographies you serve.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Input placeholder="Titles (comma-separated)" aria-label="Titles" />
            <Input placeholder="Industries" aria-label="Industries" />
            <Input placeholder="Company size band" aria-label="Company size" />
            <Input placeholder="Geographies" aria-label="Geographies" />
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
