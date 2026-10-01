import { Metadata } from 'next';
import { AnalyticsShellClient } from '../../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Job funnel' };

export default function JobFunnelAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="funnel/job" title="Job funnel" />;
}
