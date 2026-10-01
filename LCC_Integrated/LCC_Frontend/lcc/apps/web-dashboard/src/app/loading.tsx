import { LoadingSkeleton } from '@lcc/ui';

export default function GlobalLoading(): React.ReactElement {
  return (
    <div className="flex min-h-screen items-center justify-center p-8">
      <LoadingSkeleton rows={6} height="h-12" className="max-w-md" />
    </div>
  );
}
