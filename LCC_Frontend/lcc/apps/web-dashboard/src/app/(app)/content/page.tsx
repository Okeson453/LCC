import { Metadata } from 'next';
import { ContentCalendar } from './_components/content-calendar-client';

export const metadata: Metadata = { title: 'Content calendar' };

export default function ContentPage(): React.ReactElement {
  return <ContentCalendar />;
}
