'use client';

import { useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';
import { useUpdateGoalMode } from '@/lib/api/mutations/use-update-goal-mode';

export default function GoalsStep(): React.ReactElement {
  const router = useRouter();
  const [goal, setGoal] = useState<'job_hunting' | 'client_acquisition' | 'hybrid' | null>(null);
  const mutation = useUpdateGoalMode();

  async function onContinue() {
    if (!goal) return;
    await mutation.mutateAsync({ goal_mode: goal });
    router.push('/onboarding/audit');
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Step 4: Goal mode</CardTitle>
        <CardDescription>
          This shapes what the AI optimizes for. You can change it later in Settings.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <fieldset className="space-y-2">
          <legend className="sr-only">Choose goal mode</legend>
          {(['job_hunting', 'client_acquisition', 'hybrid'] as const).map((mode) => (
            <label
              key={mode}
              className="flex cursor-pointer items-start gap-3 rounded-md border p-3 transition-colors hover:bg-muted/40"
            >
              <input
                type="radio"
                name="goal"
                value={mode}
                checked={goal === mode}
                onChange={() => setGoal(mode)}
                className="mt-1"
              />
              <div>
                <p className="text-sm font-medium">
                  {mode === 'job_hunting' && 'Job hunting'}
                  {mode === 'client_acquisition' && 'Client acquisition'}
                  {mode === 'hybrid' && 'Hybrid'}
                </p>
                <p className="text-xs text-muted-foreground">
                  {mode === 'job_hunting' && 'Optimize content for hiring managers; emphasize experience, skills, projects.'}
                  {mode === 'client_acquisition' && 'Optimize for lead-gen; emphasize outcomes, case studies, ICP.'}
                  {mode === 'hybrid' && 'Balanced — the AI prioritizes based on context.'}
                </p>
              </div>
            </label>
          ))}
        </fieldset>

        <footer className="flex justify-between">
          <Button asChild variant="ghost">
            <Link href="/onboarding/kb/review">Back</Link>
          </Button>
          <Button
            variant="default"
            onClick={() => void onContinue()}
            type="button"
            disabled={!goal || mutation.isPending}
          >
            {mutation.isPending ? 'Saving…' : 'Continue'}
          </Button>
        </footer>
      </CardContent>
    </Card>
  );
}
