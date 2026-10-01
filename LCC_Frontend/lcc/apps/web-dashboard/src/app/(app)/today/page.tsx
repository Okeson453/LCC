import { Metadata } from 'next';
import { TodayPageClient } from './_components/today-page-client';

export const metadata: Metadata = { title: 'Today' };

export default function TodayPage(): React.ReactElement {
  return <TodayPageClient />;
}
