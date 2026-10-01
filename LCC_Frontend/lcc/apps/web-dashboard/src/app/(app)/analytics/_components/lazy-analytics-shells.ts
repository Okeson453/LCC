import type { Metadata } from 'next';

export type AnalyticsShellProps = { title: string; subtitle: string };

export const AnalyticsPageShells = [
  'content', 'profile', 'network', 'outreach',
  'funnel/job', 'funnel/client', 'account-health',
  'digest/weekly', 'digest/monthly',
];
