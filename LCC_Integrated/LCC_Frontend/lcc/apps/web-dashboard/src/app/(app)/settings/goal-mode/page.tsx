import { Metadata } from 'next';
import { Card, CardContent, CardDescription, CardHeader, CardTitle, RadioGroup, RadioGroupItem, Label } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'Goal mode' };

export default function GoalModePage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Goal mode</h1>
          <p className="text-sm text-muted-foreground">Pick the primary objective. Most actions adapt to this.</p>
        </header>
        <Card>
          <CardHeader>
            <CardTitle>Mode</CardTitle>
            <CardDescription>You can change this any time.</CardDescription>
          </CardHeader>
          <CardContent>
            <RadioGroup defaultValue="job">
              <div className="flex items-center space-x-2"><RadioGroupItem value="job" id="job" /><Label htmlFor="job">Job search</Label></div>
              <div className="flex items-center space-x-2"><RadioGroupItem value="client" id="client" /><Label htmlFor="client">Client acquisition</Label></div>
              <div className="flex items-center space-x-2"><RadioGroupItem value="thought_leadership" id="thought" /><Label htmlFor="thought">Thought leadership</Label></div>
              <div className="flex items-center space-x-2"><RadioGroupItem value="deal_flow" id="deal_flow" /><Label htmlFor="deal_flow">Deal flow</Label></div>
            </RadioGroup>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
