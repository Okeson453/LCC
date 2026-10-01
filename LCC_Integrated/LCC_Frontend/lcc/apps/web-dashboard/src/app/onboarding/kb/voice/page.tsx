import Link from 'next/link';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Alert, AlertDescription, AlertTitle } from '@lcc/ui';

export default function VoiceStep(): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Step 2: Voice samples</CardTitle>
        <CardDescription>
          We&apos;ll analyze your past LinkedIn posts (with your consent) to learn your
          authentic voice. This is what gives your drafts their tone.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Alert variant="warning">
          <AlertTitle>LinkedIn consent required</AlertTitle>
          <AlertDescription>
            We will request <code className="font-mono text-xs">w_member_social</code> read
            access from LinkedIn to fetch your last 50 posts. You can revoke access at any
            time in Settings → OAuth.
          </AlertDescription>
        </Alert>

        <div className="rounded-md border bg-muted/40 p-4">
          <p className="text-sm">Or paste 5–10 posts manually:</p>
          <textarea
            rows={6}
            placeholder="Paste each post on a new line…"
            className="mt-2 w-full rounded-md border bg-background px-3 py-2 text-sm"
            aria-label="Manual voice sample pastes"
          />
        </div>

        <footer className="flex justify-between">
          <Button asChild variant="ghost">
            <Link href="/onboarding/kb">Back</Link>
          </Button>
          <Button asChild variant="default">
            <Link href="/onboarding/kb/review">Continue</Link>
          </Button>
        </footer>
      </CardContent>
    </Card>
  );
}
