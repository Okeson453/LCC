import type { Metadata } from 'next';
import dynamic from 'next/dynamic';

// Lazy-load CopilotRoot so the (app) layout stays under the bundle budget.
const CopilotRoot = dynamic(() => import('@/components/copilot/CopilotRoot').then((m) => m.CopilotRoot), {
  ssr: false,
  loading: () => <p className="p-6 text-sm text-muted-foreground">Loading copilot…</p>,
});

export const metadata: Metadata = { title: 'Copilot' };

export default function CopilotPage(): React.ReactElement {
  return (
    <div className="flex h-[calc(100vh-4rem)] flex-col overflow-hidden">
      <CopilotRoot fullPage />
    </div>
  );
}
