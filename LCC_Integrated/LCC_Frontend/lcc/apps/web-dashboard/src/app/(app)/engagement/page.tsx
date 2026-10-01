import { Metadata } from 'next';
import { EngagementClient } from './_components/engagement-client';

export const metadata: Metadata = { title: 'Engagement' };

export default function EngagementPage(): React.ReactElement {
  return <EngagementClient />;
}
