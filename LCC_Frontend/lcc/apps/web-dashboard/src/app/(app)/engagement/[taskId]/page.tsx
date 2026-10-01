import { Metadata } from 'next';
import { ReplyClient } from './_components/reply-client';

export const metadata: Metadata = { title: 'Reply' };

export default function ReplyPage({ params }: { params: { taskId: string } }): React.ReactElement {
  return <ReplyClient taskId={params.taskId} />;
}
