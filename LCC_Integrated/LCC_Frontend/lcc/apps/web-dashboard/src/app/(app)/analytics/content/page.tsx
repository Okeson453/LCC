import { Metadata } from 'next';
import { AnalyticsShellClient } from '../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Content performance' };

export default function ContentAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="content" title="Content performance" />;
}
