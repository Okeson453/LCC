import { Metadata } from 'next';
import { ApprovalDetailClient } from './_components/approval-detail-client';

export const metadata: Metadata = { title: 'Approval detail' };

export default function ApprovalDetailPage({
  params,
}: {
  params: { approvalId: string };
}): React.ReactElement {
  return <ApprovalDetailClient approvalId={params.approvalId} />;
}
