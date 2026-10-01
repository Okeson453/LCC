/**
 * Onboarding layout — wizard, no sidebar.
 */

import Link from 'next/link';

const STEPS = [
  { id: 'kb', label: 'Knowledge base', href: '/onboarding/kb' },
  { id: 'voice', label: 'Voice samples', href: '/onboarding/kb/voice' },
  { id: 'review', label: 'Review KB', href: '/onboarding/kb/review' },
  { id: 'goals', label: 'Goal mode', href: '/onboarding/goals' },
  { id: 'audit', label: 'Profile audit', href: '/onboarding/audit' },
  { id: 'done', label: 'First edits', href: '/onboarding/done' },
];

export default function OnboardingLayout({ children }: { children: React.ReactNode }): React.ReactElement {
  return (
    <div className="min-h-screen bg-background">
      <header className="border-b">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <h1 className="text-lg font-semibold">Welcome to LinkedIn Manager</h1>
          <Link href="/today" className="text-xs text-muted-foreground hover:text-foreground">
            Skip for now
          </Link>
        </div>
      </header>
      <nav aria-label="Onboarding steps" className="border-b">
        <ol className="mx-auto flex max-w-5xl items-center gap-2 overflow-x-auto px-4 py-3 text-sm">
          {STEPS.map((s, i) => (
            <li key={s.id} className="flex items-center gap-2 whitespace-nowrap">
              <span className="rounded-full bg-muted px-2 py-0.5 text-xs">{i + 1}</span>
              <Link href={s.href} className="hover:underline">
                {s.label}
              </Link>
              {i < STEPS.length - 1 ? <span aria-hidden="true" className="text-muted-foreground">→</span> : null}
            </li>
          ))}
        </ol>
      </nav>
      <main className="mx-auto max-w-5xl p-6">{children}</main>
    </div>
  );
}
