'use client';

/**
 * ApprovalDialog — the universal approval surface.
 *
 * Used by Web Dashboard, Mobile PWA, and Browser Extension sidepanel. Renders
 * an action preview, target, KB grounding citations, risk tier, and
 * tier-specific UX rules. Calls the consumer's `onApprove` / `onReject`
 * callbacks (which MUST route through `@lcc/approval-gate` hooks, not
 * directly to the integration endpoint).
 *
 * Non-negotiables enforced:
 *   - KB citations must be present (assertCitationsPresent)
 *   - Tier 5 actions do not respond to Enter-key submit
 *   - Focus is trapped while open
 *   - Reduced-motion users see instant transitions
 */

import * as React from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Button, Textarea, cn } from '@lcc/ui';
import { tierMeta } from '@lcc/tokens';
import type {
  ApprovalActionType,
  KbCitation,
  RiskTier,
} from '@lcc/api-types';
import { ACTION_TYPE_LABELS } from '@lcc/api-types';
import { RiskTierBadge } from './RiskTierBadge';
import { KbCitationsList } from './KbCitationsList';
import { GovernanceTrace, type GovernanceEvaluation } from './GovernanceTrace';
import { GuardFailurePanel } from './GuardFailurePanel';
import { PermitTokenView } from './PermitTokenView';
import { getTierUxRule } from '../utils/tier-rules';
import { assertCitationsPresent } from '../utils/citation-grouper';

export interface ApprovalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  approvalId: string;
  actionType: ApprovalActionType;
  tier: RiskTier;
  /** Preview body (post body, message, comment, etc.). */
  preview: string;
  /** Target (person, post, opportunity, profile field). */
  targetLabel: string;
  /** Editable preview (Tier 3+). */
  editablePreview?: boolean;
  /** KB citations — required. */
  kbRefs: readonly KbCitation[];
  /** Trace ID for support. */
  traceId: string;
  /** Idempotency key — supplied by caller. */
  idempotencyKey: string;
  /** Initial preview value when editing. */
  initialPreview?: string;
  /** Tier 5 typed-confirmation phrase (e.g., "Apply"). */
  typedConfirmationPhrase?: string;
  /** Whether there's prior interaction (Tier 4 warning). */
  hasPriorInteraction?: boolean;
  /** Submitting state — disables buttons. */
  isSubmitting?: boolean;
  /** Optional governance result (after submit). */
  evaluation?: GovernanceEvaluation | null;
  /** Approve callback. */
  onApprove: (input: {
    decision: 'approve';
    editedPreview?: string;
    comment?: string;
    approvalId: string;
    idempotencyKey: string;
    traceId: string;
  }) => void | Promise<void>;
  /** Reject callback. */
  onReject: (input: {
    decision: 'reject';
    comment?: string;
    approvalId: string;
    idempotencyKey: string;
    traceId: string;
  }) => void | Promise<void>;
  /** Edit-and-retry callback (after a denial). */
  onEditAndRetry?: () => void;
  className?: string;
}

export function ApprovalDialog({
  open,
  onOpenChange,
  approvalId,
  actionType,
  tier,
  preview,
  targetLabel,
  editablePreview = false,
  kbRefs,
  traceId,
  idempotencyKey,
  initialPreview,
  typedConfirmationPhrase,
  hasPriorInteraction = true,
  isSubmitting = false,
  evaluation,
  onApprove,
  onReject,
  onEditAndRetry,
  className,
}: ApprovalDialogProps): React.ReactElement {
  const [edited, setEdited] = React.useState(initialPreview ?? preview);
  const [comment, setComment] = React.useState('');
  const [typedConfirm, setTypedConfirm] = React.useState('');
  const uxRule = getTierUxRule(tier);
  const isDenied = evaluation ? !evaluation.permit : false;

  // Audit hook — empty citation list is a violation.
  React.useEffect(() => {
    if (open) assertCitationsPresent(kbRefs, `ApprovalDialog(${approvalId})`);
  }, [open, kbRefs, approvalId]);

  // Reset edited state when dialog opens.
  React.useEffect(() => {
    if (open) {
      setEdited(initialPreview ?? preview);
      setComment('');
      setTypedConfirm('');
    }
  }, [open, initialPreview, preview]);

  // Handle keyboard submission — disabled for Tier 5 to force explicit click.
  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key !== 'Enter') return;
    if (e.target instanceof HTMLTextAreaElement) return;
    if (!uxRule.allowEnterSubmit) return;
    e.preventDefault();
    handleApprove();
  };

  const typedConfirmOk =
    !uxRule.requireTypedConfirmation ||
    (typedConfirmationPhrase !== undefined &&
      typedConfirm.trim().toLowerCase() === typedConfirmationPhrase.trim().toLowerCase());

  const canApprove =
    !isSubmitting &&
    !isDenied &&
    typedConfirmOk &&
    (uxRule.editableMessage ? edited.trim().length > 0 : true);

  function handleApprove() {
    if (!canApprove) return;
    void onApprove({
      decision: 'approve',
      editedPreview: uxRule.editableMessage ? edited : undefined,
      comment: comment.trim() || undefined,
      approvalId,
      idempotencyKey,
      traceId,
    });
  }

  function handleReject() {
    void onReject({
      decision: 'reject',
      comment: comment.trim() || undefined,
      approvalId,
      idempotencyKey,
      traceId,
    });
  }

  const actionLabel = ACTION_TYPE_LABELS[actionType];

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            'fixed inset-0 z-50 bg-background/80 backdrop-blur-sm',
            'data-[state=open]:animate-in data-[state=closed]:animate-out',
            'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
          )}
        />
        <Dialog.Content
          onKeyDown={handleKeyDown}
          aria-labelledby={`approval-${approvalId}-title`}
          aria-describedby={`approval-${approvalId}-desc`}
          className={cn(
            'fixed left-1/2 top-1/2 z-50 grid w-full max-w-2xl -translate-x-1/2 -translate-y-1/2',
            'gap-4 border bg-background p-6 shadow-lg sm:rounded-lg',
            'max-h-[90vh] overflow-y-auto',
            className,
          )}
        >
          <Dialog.Title
            id={`approval-${approvalId}-title`}
            className="flex items-center gap-2 text-lg font-semibold"
          >
            <span>{actionLabel}</span>
            <RiskTierBadge tier={tier} />
          </Dialog.Title>
          <Dialog.Description
            id={`approval-${approvalId}-desc`}
            className="text-sm text-muted-foreground"
          >
            Target: {targetLabel}
          </Dialog.Description>

          {/* Preview */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium">Preview</h4>
            {editablePreview && uxRule.editableMessage ? (
              <Textarea
                value={edited}
                onChange={(e) => setEdited(e.target.value)}
                rows={6}
                aria-label="Editable preview"
                disabled={isSubmitting}
              />
            ) : (
              <blockquote
                className="rounded-md border bg-muted/40 p-3 text-sm"
                aria-label="Action preview"
              >
                {preview}
              </blockquote>
            )}
          </div>

          {/* Tier 4 warning when no prior interaction */}
          {tier === 4 && !hasPriorInteraction ? (
            <div
              role="alert"
              className="rounded-md border border-warning/40 bg-warning/10 p-3 text-sm"
            >
              <strong>Heads up:</strong> You have no prior interaction with this contact.
              Consider adding a personal note before sending.
            </div>
          ) : null}

          {/* KB citations — mandatory */}
          <KbCitationsList citations={kbRefs} context={`ApprovalDialog(${approvalId})`} />

          {/* Optional typed confirmation (Tier 5) */}
          {uxRule.requireTypedConfirmation && typedConfirmationPhrase ? (
            <div className="space-y-2">
              <label
                htmlFor={`approval-${approvalId}-confirm`}
                className="block text-sm font-medium"
              >
                Type <span className="font-mono">{typedConfirmationPhrase}</span> to confirm
              </label>
              <input
                id={`approval-${approvalId}-confirm`}
                type="text"
                value={typedConfirm}
                onChange={(e) => setTypedConfirm(e.target.value)}
                className="w-full rounded-md border bg-background px-3 py-2 text-sm"
                aria-required="true"
                disabled={isSubmitting}
                placeholder={typedConfirmationPhrase}
              />
            </div>
          ) : null}

          {/* Optional comment */}
          <div className="space-y-2">
            <label
              htmlFor={`approval-${approvalId}-comment`}
              className="block text-sm font-medium"
            >
              Comment (optional)
            </label>
            <Textarea
              id={`approval-${approvalId}-comment`}
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              rows={2}
              disabled={isSubmitting}
            />
          </div>

          {/* Governance evaluation result (post-submit) */}
          {evaluation ? (
            <GovernanceTrace evaluation={evaluation} traceId={traceId} />
          ) : null}

          {/* Denial panel */}
          {isDenied && evaluation ? (
            <GuardFailurePanel
              guardName={evaluation.failedGuard ?? 'unknown'}
              reason={evaluation.reason ?? 'Unknown denial'}
              traceId={traceId}
              onCancel={() => onOpenChange(false)}
              onRetry={onEditAndRetry}
            />
          ) : null}

          {/* Permit token (debug-only) */}
          {evaluation?.permitToken ? (
            <PermitTokenView
              permitToken={evaluation.permitToken}
              expiresAt={evaluation.expiresAt}
              traceId={traceId}
            />
          ) : null}

          {/* Footer actions */}
          <footer className="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
            <Button
              variant="ghost"
              onClick={() => onOpenChange(false)}
              type="button"
              disabled={isSubmitting}
            >
              Close
            </Button>
            <Button
              variant="outline"
              onClick={handleReject}
              type="button"
              disabled={isSubmitting}
            >
              Reject
            </Button>
            <Button
              variant="default"
              onClick={handleApprove}
              type="button"
              disabled={!canApprove}
              aria-disabled={!canApprove}
              style={{
                backgroundColor: canApprove ? tierMeta[tier].cssVar : undefined,
              }}
            >
              {isSubmitting ? 'Approving…' : `Approve & ${actionLabel.toLowerCase()}`}
            </Button>
          </footer>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
