import { Metadata } from 'next';
import { AnalyticsShellClient } from '../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Account health' };

export default function AccountHealthAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="account-health" title="Account health" />;
}
