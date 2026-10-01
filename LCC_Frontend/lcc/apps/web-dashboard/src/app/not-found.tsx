import Link from 'next/link';
import { Button } from '@lcc/ui';

export default function NotFound(): React.ReactElement {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-8 text-center">
      <h1 className="text-6xl font-bold">404</h1>
      <p className="mt-2 text-muted-foreground">This page does not exist.</p>
      <Button asChild className="mt-6" variant="outline">
        <Link href="/today">Back to Today</Link>
      </Button>
    </div>
  );
}
