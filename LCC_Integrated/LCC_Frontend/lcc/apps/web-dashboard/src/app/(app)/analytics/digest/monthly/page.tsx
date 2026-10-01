import { Metadata } from 'next';
import { AnalyticsShellClient } from '../../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Monthly digest' };

export default function MonthlyDigest(): React.ReactElement {
  return <AnalyticsShellClient metric="digest/monthly" title="Monthly digest" />;
}

