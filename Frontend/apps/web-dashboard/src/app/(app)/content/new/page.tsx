import { Metadata } from 'next';
import { ComposerClient } from './_components/composer-client';

export const metadata: Metadata = { title: 'New post' };

export default function NewContentPage(): React.ReactElement {
  return <ComposerClient />;
}
