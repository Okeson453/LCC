import { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { OpportunityDetailShell } from './_components/opportunity-detail-shell';

export const metadata: Metadata = { title: 'Opportunity' };

export default async function OpportunityDetailPage({ params }: { params: { opportunityId: string } }): Promise<React.ReactElement> {
  const id = params.opportunityId;
  if (!id) notFound();
  return <OpportunityDetailShell opportunityId={id} />;
}
