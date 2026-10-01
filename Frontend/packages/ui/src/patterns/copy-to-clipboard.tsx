'use client';

import * as React from 'react';
import { Copy, Check } from 'lucide-react';
import { cn } from '../utils/cn';

export interface CopyToClipboardProps {
  /** The text to copy. */
  text: string;
  /** Render override. Defaults to the icon button. */
  children?: React.ReactNode;
  className?: string;
  id?: string;
  ariaLabel?: string;
}

export function CopyToClipboard({
  text,
  children,
  className,
  id,
  ariaLabel = 'Copy to clipboard',
}: CopyToClipboardProps): React.ReactElement {
  const [copied, setCopied] = React.useState(false);

  async function onCopy() {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(text);
      } else {
        const ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.opacity = '0';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
      }
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      // ignore
    }
  }

  return (
    <button
      id={id}
      type="button"
      onClick={() => void onCopy()}
      className={cn(
        'inline-flex items-center gap-1 rounded-md border bg-background px-2 py-1 text-xs hover:bg-accent',
        className,
      )}
      aria-label={ariaLabel}
    >
      {children ?? (copied ? <Check className="h-3 w-3" aria-hidden="true" /> : <Copy className="h-3 w-3" aria-hidden="true" />)}
      <span className="sr-only">{ariaLabel}</span>
    </button>
  );
}
