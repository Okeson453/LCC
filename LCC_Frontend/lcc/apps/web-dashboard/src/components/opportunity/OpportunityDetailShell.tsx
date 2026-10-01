'use client';

import * as React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent, Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import { OpportunityEvidencePanel } from './OpportunityEvidencePanel';
import { OpportunityActionPlanCard } from './OpportunityActionPlanCard';
import { OpportunityStagePicker } from './OpportunityStagePicker';
import type { Opportunity, OpportunityStage } from '@lcc/api-types';

export function OpportunityDetailShell({
  opp,
  onStageChange,
}: {
  opp: Opportunity;
  onStageChange: (s: OpportunityStage) => Promise<void>;
}): React.ReactElement {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader><CardTitle className="text-sm">Stage</CardTitle></CardHeader>
        <CardContent>
          <OpportunityStagePicker value={opp.status as OpportunityStage} onChange={onStageChange} />
        </CardContent>
      </Card>

      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="evidence">Evidence</TabsTrigger>
          <TabsTrigger value="actions">Actions</TabsTrigger>
        </TabsList>
        <TabsContent value="overview">
          <Card>
            <CardHeader><CardTitle className="text-sm">Why this fits</CardTitle></CardHeader>
            <CardContent><p className="text-sm text-muted-foreground">{opp.why_fit}</p></CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="evidence">
          <OpportunityEvidencePanel evidence={opp.evidence} />
        </TabsContent>
        <TabsContent value="actions">
          <OpportunityActionPlanCard plan={opp.action_plan} actionItems={opp.action_items} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
