'use client';

import * as React from 'react';
import Link from 'next/link';
import { Button, Badge } from '@lcc/ui';
import type { ProfileSnapshot } from '@lcc/api-types';

export function ProfileHeader({ profile }: { profile: ProfileSnapshot }): React.ReactElement {
  return (
    <header className="flex items-center justify-between">
      <div>
        <h1 className="text-2xl font-bold">{profile.display_name}</h1>
        <p className="text-sm text-muted-foreground">{profile.headline}</p>
      </div>
      <div className="flex gap-2 text-sm">
        <Link href="/profile/audit" className="text-primary hover:underline">Audit</Link>
        <Link href="/profile/edits" className="text-primary hover:underline">Edits</Link>
        <Link href="/profile/history" className="text-primary hover:underline">History</Link>
        <Button asChild size="sm" variant="outline" type="button"><Link href="/today">Back</Link></Button>
      </div>
    </header>
  );
}
