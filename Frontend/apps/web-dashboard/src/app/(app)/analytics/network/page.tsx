import { Metadata } from 'next';
import { AnalyticsShellClient } from '../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Network health' };

export default function NetworkAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="network" title="Network health" />;
}
