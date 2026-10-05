/**
 * Type declarations for the shared Tailwind presets.
 *
 * `@lcc/ui` and `@lcc/tokens` both export their presets as plain CommonJS
 * (`.cjs`) so Tailwind can `require()` them at config time. That means they
 * ship no types, and TypeScript inferred a structural shape that did not match
 * Tailwind's own `Partial<Config>` — `presets` and `darkMode` came out as
 * bare arrays, so `tailwind.config.ts` failed to typecheck.
 */

declare module '@lcc/ui/tailwind-preset' {
  import type { Config } from 'tailwindcss';
  const preset: Config;
  export default preset;
}

declare module '@lcc/tokens/tailwind-preset' {
  import type { Config } from 'tailwindcss';
  const preset: Config;
  export default preset;
}

declare module '@lcc/ui/tailwind-preset.cjs' {
  import type { Config } from 'tailwindcss';
  const preset: Config;
  export = preset;
}

declare module '@lcc/tokens/tailwind-preset.cjs' {
  import type { Config } from 'tailwindcss';
  const preset: Config;
  export = preset;
}
