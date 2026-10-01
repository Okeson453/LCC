import { Metadata } from 'next';
import { ContactDetailClient } from './_components/contact-detail-client';

export const metadata: Metadata = { title: 'Contact' };

export default function ContactPage({ params }: { params: { contactId: string } }): React.ReactElement {
  return <ContactDetailClient contactId={params.contactId} />;
}
