import Link from 'next/link';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';

export default function DoneStep(): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>You&apos;re all set 🎉</CardTitle>
        <CardDescription>
          Your KB is seeded, your goal mode is set, and your first profile audit is
          running. Review the suggested edits when they&apos;re ready, or jump into Today.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2 sm:flex-row">
        <Button asChild variant="default">
          <Link href="/today">Go to Today</Link>
        </Button>
        <Button asChild variant="outline">
          <Link href="/profile">Review profile edits</Link>
        </Button>
      </CardContent>
    </Card>
  );
}
