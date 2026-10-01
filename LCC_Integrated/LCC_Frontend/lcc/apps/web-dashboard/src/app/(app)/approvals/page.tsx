import { Metadata } from 'next';
import { ApprovalsPageClient } from './_components/approvals-page-client';

export const metadata: Metadata = { title: 'Approvals' };

export default function ApprovalsPage(): React.ReactElement {
  return <ApprovalsPageClient />;
}
