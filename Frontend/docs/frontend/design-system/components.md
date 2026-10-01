# Component catalog

`packages/ui` exports an inventory of accessible primitives and patterns built on
shadcn-style Radix roots and Tailwind.

## Primitives

`Button`, `Input`, `Textarea`, `Checkbox`, `Switch`, `RadioGroup`, `Select`,
`Slider`, `Calendar`, `Command`, `Dialog`, `Sheet`, `Tooltip`, `Popover`,
`DropdownMenu`, `Toast`, `Tabs`, `Card`, `Badge`, `Avatar`, `Pagination`,
`Table`, `Form`, `Toggle`, `ToggleGroup`.

## Patterns

- `<EmptyState>` for lists with no data.
- `<ErrorState>` with retry CTA.
- `<LoadingSkeleton>` with aria-busy semantics.
- `<ConfirmDialog>` for irreversible actions (e.g., delete draft).
- `<DataTable>` reads from TanStack Query + sorts/filters/paginates.
- `<Alert variant={destructive|warning|info|success}>`.

## Vendored layers

- Approval-gate bundled at top-level (`@lcc/approval-gate`).
- Compliance-state banners (`@lcc/compliance-state`).

## Composition rules

- A package primitive can wrap another primitive; never inline a primitive
  inside another primitive unless both live in `@lcc/ui`.
- All input controls must be wrapped in `<Form>` when used in mutations.

## Testing

- Renderers live in `@lcc/test-utils/render` with theme + i18n providers.
- Coverage thresholds: lines/statements 70%, branches 65%, functions 70%.
