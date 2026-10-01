'use client';

import { Metadata } from 'next';
import { Card, CardContent, CardHeader, CardTitle, Switch } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';
import { useState } from 'react';

export const metadata: Metadata = { title: 'Notifications' };

export default function NotificationsPage(): React.ReactElement {
  const [email, setEmail] = useState(true);
  const [push, setPush] = useState(false);
  const [sms, setSms] = useState(false);

  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <header>
          <h1 className="text-2xl font-bold">Notifications</h1>
        </header>
        <Card>
          <CardHeader><CardTitle>Channels</CardTitle></CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between">
              <label htmlFor="email" className="text-sm">Email</label>
              <Switch id="email" checked={email} onCheckedChange={setEmail} />
            </div>
            <div className="flex items-center justify-between">
              <label htmlFor="push" className="text-sm">Push</label>
              <Switch id="push" checked={push} onCheckedChange={setPush} />
            </div>
            <div className="flex items-center justify-between">
              <label htmlFor="sms" className="text-sm">SMS</label>
              <Switch id="sms" checked={sms} onCheckedChange={setSms} />
            </div>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
