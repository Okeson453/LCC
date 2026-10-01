'use client';

import * as React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent, Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import { ContactStagePicker } from './ContactStagePicker';
import { ContactTimeline } from './ContactTimeline';
import type { Contact, RelationshipStage, Interaction } from '@lcc/api-types';

export interface ContactDetailShellProps {
  contact: Contact;
  interactions: Interaction[];
  onStageChange: (stage: RelationshipStage) => Promise<void>;
  onLogInteraction: (input: { kind: string; occurred_at: string; direction: 'inbound' | 'outbound' }) => Promise<void>;
}

export function ContactDetailShell({ contact, interactions, onStageChange, onLogInteraction }: ContactDetailShellProps): React.ReactElement {
  const [stage, setStage] = React.useState<RelationshipStage>(contact.stage);
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader><CardTitle className="text-sm">Stage</CardTitle></CardHeader>
        <CardContent className="flex gap-2">
          <ContactStagePicker value={stage} onChange={async (s) => { setStage(s); await onStageChange(s); }} />
        </CardContent>
      </Card>

      <Tabs defaultValue="timeline">
        <TabsList>
          <TabsTrigger value="timeline">Timeline</TabsTrigger>
          <TabsTrigger value="profile">Profile</TabsTrigger>
        </TabsList>
        <TabsContent value="timeline">
          <ContactTimeline interactions={interactions} onLog={onLogInteraction} />
        </TabsContent>
        <TabsContent value="profile">
          <Card>
            <CardHeader><CardTitle className="text-sm">Profile</CardTitle></CardHeader>
            <CardContent>
              <p className="text-xs text-muted-foreground">{contact.headline}</p>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
