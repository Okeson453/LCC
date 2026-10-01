import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'Timezone' };

const TIMEZONES = ['UTC', 'Europe/Berlin', 'Europe/Madrid', 'Europe/London', 'America/New_York', 'America/Los_Angeles', 'America/Chicago'];

export default function TimezonePage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Timezone</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>Your timezone</CardTitle></CardHeader>
          <CardContent>
            <Select defaultValue="UTC">
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                {TIMEZONES.map((tz) => (
                  <SelectItem key={tz} value={tz}>{tz}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
