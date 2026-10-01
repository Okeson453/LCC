'use client';

/**
 * KbCitationsList — mandatory grounding citation list.
 *
 * Per Non-Negotiable §5 (axiom 5): every artifact shown for approval must
 * display its KB grounding citations. This component renders the list
 * grouped by category. Empty citation lists trigger an audit warning.
 */

import * as React from 'react';
import type { KbCitation } from '@lcc/api-types';
import { groupCitations, assertCitationsPresent } from '../utils/citation-grouper';

export interface KbCitationsListProps {
  citations: readonly KbCitation[];
  /** Optional context label for the audit warning. */
  context?: string;
  /** Show excerpt text. Defaults to true. */
  showExcerpt?: boolean;
  className?: string;
  id?: string;
}

export function KbCitationsList({
  citations,
  context = 'KbCitationsList',
  showExcerpt = true,
  className,
  id,
}: KbCitationsListProps): React.ReactElement {
  // Audit hook: empty citation list is a violation. Warn in dev, log in prod.
  React.useEffect(() => {
    assertCitationsPresent(citations, context);
  }, [citations, context]);

  const groups = groupCitations(citations);

  if (groups.length === 0) {
    return (
      <div
        id={id}
        role="alert"
        aria-live="polite"
        className={[
          'rounded-md border border-dashed border-destructive/50 bg-destructive/5 p-3 text-sm text-destructive',
          className,
        ]
          .filter(Boolean)
          .join(' ')}
      >
        <strong>No KB grounding citations.</strong> Every approval must cite the KB
        records that ground the action. Contact the engineering team if this is unexpected.
      </div>
    );
  }

  return (
    <section
      id={id}
      aria-label="KB grounding citations"
      className={['rounded-md border bg-muted/30 p-3', className].filter(Boolean).join(' ')}
    >
      <header className="mb-2 flex items-center justify-between">
        <h4 className="text-sm font-medium">
          Grounding ({citations.length} {citations.length === 1 ? 'citation' : 'citations'})
        </h4>
        <span className="text-xs text-muted-foreground" aria-hidden="true">
          Required by policy
        </span>
      </header>
      <ul className="space-y-3">
        {groups.map((group) => (
          <li key={group.category}>
            <h5 className="mb-1 text-xs font-medium uppercase tracking-wide text-muted-foreground">
              {group.label}
            </h5>
            <ul className="space-y-2">
              {group.citations.map((c) => (
                <li
                  key={c.recordId}
                  className="rounded-md border bg-background p-2 text-sm"
                >
                  <div className="flex items-start justify-between gap-2">
                    <span className="font-medium">{c.title}</span>
                    {c.url ? (
                      <a
                        href={c.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-xs text-primary underline-offset-2 hover:underline"
                      >
                        Open
                      </a>
                    ) : null}
                  </div>
                  {showExcerpt && c.excerpt ? (
                    <p className="mt-1 line-clamp-3 text-xs text-muted-foreground">
                      {c.excerpt}
                    </p>
                  ) : null}
                </li>
              ))}
            </ul>
          </li>
        ))}
      </ul>
    </section>
  );
}
