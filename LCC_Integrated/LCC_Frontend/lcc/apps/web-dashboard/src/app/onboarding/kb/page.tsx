import Link from 'next/link';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';

export default function KbStep(): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Step 1: Knowledge base</CardTitle>
        <CardDescription>
          Upload your resume, add portfolio links, and seed your KB. The AI uses these to
          ground every artifact it produces.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <section>
          <h2 className="text-sm font-medium">Resume / CV</h2>
          <p className="text-xs text-muted-foreground">PDF or DOCX, max 10 MB</p>
          <input
            type="file"
            accept=".pdf,.docx,.txt"
            aria-label="Upload resume"
            className="mt-2 w-full rounded-md border bg-background px-3 py-2 text-sm"
          />
        </section>

        <section>
          <h2 className="text-sm font-medium">Portfolio links</h2>
          <p className="text-xs text-muted-foreground">Add up to 10 links (one per line)</p>
          <textarea
            rows={4}
            placeholder="https://yoursite.example"
            className="mt-2 w-full rounded-md border bg-background px-3 py-2 text-sm font-mono"
            aria-label="Portfolio links"
          />
        </section>

        <footer className="flex justify-end gap-2">
          <Button asChild variant="ghost">
            <Link href="/today">Skip</Link>
          </Button>
          <Button asChild variant="default">
            <Link href="/onboarding/kb/voice">Continue</Link>
          </Button>
        </footer>
      </CardContent>
    </Card>
  );
}
