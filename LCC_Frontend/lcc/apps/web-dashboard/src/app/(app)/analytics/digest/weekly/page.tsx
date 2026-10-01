import { Metadata } from 'next';
import { AnalyticsShellClient } from '../../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Weekly digest' };

export default function WeeklyDigest(): React.ReactElement {
  return <AnalyticsShellClient metric="digest/weekly" title="Weekly digest" />;
}

