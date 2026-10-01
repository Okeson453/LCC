import { Metadata } from 'next';
import { AnalyticsShellClient } from '../../_components/analytics-shell-client';

export const metadata: Metadata = { title: 'Client funnel' };

export default function ClientFunnelAnalytics(): React.ReactElement {
  return <AnalyticsShellClient metric="funnel/client" title="Client funnel" />;
}
