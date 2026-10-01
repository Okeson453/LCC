import { Metadata } from 'next';
import { DraftEditorClient } from './_components/draft-editor-client';

export const metadata: Metadata = { title: 'Draft editor' };

export default function ContentItemPage({ params }: { params: { id: string } }): React.ReactElement {
  return <DraftEditorClient contentId={params.id} />;
}
