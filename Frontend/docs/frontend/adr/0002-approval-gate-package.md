# ADR 0002: Approval gate as a separate package

## Status

Accepted.

## Decision

The approval gate is a stable, cross-app surface. Move it to `@lcc/approval-gate`.

## Why

- Renders identically in web dashboard, mobile PWA, and extension sidepanel.
- Tier rules are referenced by tests across packages.
- Decoupling from `app/(app)` makes it reusable inside the extension popup and the
  PWA.

## Consequences

- Heavy `<ApprovalDialog>` couples to `@lcc/ui`. We accept this dependency because
  primitive button/dialog components are already in `@lcc/ui`.
- Versioning of the package is monorepo-aligned (no separate semver).
