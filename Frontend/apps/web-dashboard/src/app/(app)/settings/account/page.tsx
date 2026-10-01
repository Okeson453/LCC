import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { signOut } from 'next-auth/react';
import Link from 'next/link';

export const metadata: Metadata = { title: 'Account' };

export default function AccountPage(): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Account</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>Session</CardTitle></CardHeader>
          <CardContent className="flex gap-2">
            <Button asChild variant="outline"><Link href="/api/auth/linkedin/start">Reconnect LinkedIn</Link></Button>
            <Button onClick={() => void signOut()} variant="destructive" type="button">Sign out</Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
