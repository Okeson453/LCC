import { Metadata } from 'next';
import { AnalyticsShellClient } from '../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Profile performance' };

export default function ProfileAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="profile" title="Profile performance" />;
}
