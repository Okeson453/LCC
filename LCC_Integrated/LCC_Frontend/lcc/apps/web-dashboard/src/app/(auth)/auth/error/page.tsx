/**
 * /auth/error — display OAuth errors.
 */

import Link from 'next/link';
import { Button } from '@lcc/ui';

const REASON_LABELS: Record<string, string> = {
  missing_params: 'OAuth was missing required parameters.',
  server_error: 'The authorization server is temporarily unavailable.',
  access_denied: 'You declined the LinkedIn authorization.',
};

export default function AuthErrorPage({
  searchParams,
}: {
  searchParams: { reason?: string };
}): React.ReactElement {
  const reason = searchParams.reason ?? 'unknown';
  const message = REASON_LABELS[reason] ?? 'Authorization failed. Please try again.';

  return (
    <div className="w-full max-w-md space-y-4 rounded-lg border bg-card p-6 text-center">
      <h1 className="text-xl font-semibold">Sign-in failed</h1>
      <p className="text-sm text-muted-foreground">{message}</p>
      <Button asChild variant="default" className="w-full">
        <Link href="/auth/linkedin/start">Try again</Link>
      </Button>
    </div>
  );
}
