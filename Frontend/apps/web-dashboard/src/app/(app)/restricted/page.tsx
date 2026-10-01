import { Metadata } from 'next';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';
import Link from 'next/link';

export const metadata: Metadata = { title: 'Account restricted' };

export default function RestrictedPage(): React.ReactElement {
  return (
    <div className="flex min-h-[60vh] items-center justify-center p-6">
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>Account restricted</CardTitle>
          <CardDescription>
            The Compliance Governor has paused your account. New actions are blocked
            until the restriction is cleared.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Existing scheduled actions will continue. To resolve, contact support.
          </p>
          <div className="flex gap-2">
            <Button asChild variant="default">
              <Link href="mailto:support@okeson.example">Contact support</Link>
            </Button>
            <Button asChild variant="outline">
              <Link href="/today">Back to dashboard</Link>
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
