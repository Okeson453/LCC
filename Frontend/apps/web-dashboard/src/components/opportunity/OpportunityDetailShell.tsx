'use client';

import * as React from 'react';
import { Tabs, TabsList, TabsTrigger, TabsContent, Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import { OpportunityEvidencePanel } from './OpportunityEvidencePanel';
import { OpportunityActionPlanCard } from './OpportunityActionPlanCard';
import { OpportunityStagePicker } from './OpportunityStagePicker';
import type {
  ActionItem,
  Opportunity,
  OpportunityEvidence,
  OpportunityStatus,
} from '@lcc/api-types';

export function OpportunityDetailShell({
  opp,
  evidence,
  actionItems,
  onStageChange,
}: {
  opp: Opportunity;
  /** Evidence backing the opportunity. Not part of the wire `Opportunity`. */
  evidence?: OpportunityEvidence[];
  /** Action items derived from the plan. Not part of the wire `Opportunity`. */
  actionItems?: ActionItem[];
  onStageChange: (s: OpportunityStatus) => Promise<void>;
}): React.ReactElement {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader><CardTitle className="text-sm">Stage</CardTitle></CardHeader>
        <CardContent>
          <OpportunityStagePicker value={opp.status} onChange={onStageChange} />
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
            <CardContent>
              <p className="text-sm text-muted-foreground">{opp.title} at {opp.company}</p>
              <p className="mt-2 text-xs text-muted-foreground">
                Fit score {opp.fit_score} · plan: {opp.action_plan}
              </p>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="evidence">
          <OpportunityEvidencePanel evidence={evidence} />
        </TabsContent>
        <TabsContent value="actions">
          <OpportunityActionPlanCard plan={opp.action_plan} actionItems={actionItems} />
        </TabsContent>
      </Tabs>
    </div>
  );
}
