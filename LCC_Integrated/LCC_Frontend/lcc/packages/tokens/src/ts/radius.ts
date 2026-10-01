export const radius = {
  sm: '0.25rem',
  md: '0.375rem',
  lg: '0.5rem',
  full: '9999px',
} as const;

export type RadiusKey = keyof typeof radius;
