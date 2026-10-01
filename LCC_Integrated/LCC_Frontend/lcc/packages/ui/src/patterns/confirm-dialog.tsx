'use client';

import * as React from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../primitives/dialog';
import { Button } from '../primitives/button';

export interface ConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  confirmLabel?: string;
  cancelLabel?: string;
  destructive?: boolean;
  /** Confirmation phrase the user must type (destructive only). */
  requireTyped?: string;
  isSubmitting?: boolean;
  onConfirm: () => void | Promise<void>;
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel = 'Confirm',
  cancelLabel = 'Cancel',
  destructive = false,
  requireTyped,
  isSubmitting = false,
  onConfirm,
}: ConfirmDialogProps): React.ReactElement {
  const [typed, setTyped] = React.useState('');

  React.useEffect(() => {
    if (open) setTyped('');
  }, [open]);

  const typedOk = !requireTyped || typed.trim() === requireTyped;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        {requireTyped ? (
          <div className="space-y-2">
            <label htmlFor="confirm-typed" className="text-sm font-medium">
              Type <span className="font-mono">{requireTyped}</span> to confirm
            </label>
            <input
              id="confirm-typed"
              type="text"
              value={typed}
              onChange={(e) => setTyped(e.target.value)}
              className="w-full rounded-md border bg-background px-3 py-2 text-sm"
              aria-required="true"
              disabled={isSubmitting}
              placeholder={requireTyped}
              autoFocus
            />
          </div>
        ) : null}
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange(false)} type="button" disabled={isSubmitting}>
            {cancelLabel}
          </Button>
          <Button
            variant={destructive ? 'destructive' : 'default'}
            onClick={() => void onConfirm()}
            type="button"
            disabled={isSubmitting || !typedOk}
          >
            {isSubmitting ? 'Working…' : confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
