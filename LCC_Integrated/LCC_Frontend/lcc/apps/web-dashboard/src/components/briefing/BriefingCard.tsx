'use client';

import * as React from 'react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle } from '@lcc/ui';
import { ChevronRight } from 'lucide-react';

export interface BriefingCardProps {
  title: string;
  subtitle?: string;
  href?: string;
  children: React.ReactNode;
}

export function BriefingCard({ title, subtitle, href, children }: BriefingCardProps): React.ReactElement {
  const inner = (
    <Card className={href ? 'transition hover:border-primary/40' : undefined}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{title}</CardTitle>
          {href ? <ChevronRight className="h-4 w-4 text-muted-foreground" aria-hidden="true" /> : null}
        </div>
        {subtitle ? <p className="text-xs text-muted-foreground">{subtitle}</p> : null}
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
  return href ? <Link href={href}>{inner}</Link> : inner;
}
