import { Metadata } from 'next';
import { AnalyticsShellClient } from '../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Outreach metrics' };

export default function OutreachAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="outreach" title="Outreach metrics" />;
}
