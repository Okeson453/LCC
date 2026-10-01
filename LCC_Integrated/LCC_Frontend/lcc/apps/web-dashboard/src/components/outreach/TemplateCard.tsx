'use client';

import * as React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Badge } from '@lcc/ui';

export interface OutreachTemplate {
  id: string;
  name: string;
  persona: string;
  body_preview: string;
}

export function TemplateCard({ template }: { template: OutreachTemplate }): React.ReactElement {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">{template.name}</CardTitle>
          <Badge variant="outline">{template.persona}</Badge>
        </div>
      </CardHeader>
      <CardContent>
        <p className="line-clamp-3 text-xs text-muted-foreground">{template.body_preview}</p>
      </CardContent>
    </Card>
  );
}
