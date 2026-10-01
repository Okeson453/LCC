import { Metadata } from 'next';
import Link from 'next/link';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';
import { ComplianceGate } from '@lcc/compliance-state';

export const metadata: Metadata = { title: 'Apply' };

export default function ApplyPage({ params }: { params: { opportunityId: string } }): React.ReactElement {
  return (
    <ComplianceGate>
      <div className="mx-auto max-w-2xl space-y-6 p-6">
        <Card>
          <CardHeader>
            <CardTitle>Apply</CardTitle>
            <CardDescription>
              Use the proposal builder to draft a tailored application. No LinkedIn
              URL or contact info is submitted to the target without your explicit
              approval.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button asChild>
              <Link href={`/opportunities/${params.opportunityId}/proposal`}>Open proposal builder</Link>
            </Button>
          </CardContent>
        </Card>
      </div>
    </ComplianceGate>
  );
}
