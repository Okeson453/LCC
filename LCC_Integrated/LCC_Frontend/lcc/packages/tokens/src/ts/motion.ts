/**
 * Motion tokens. Read from CSS variables at runtime so that the user's
 * `prefers-reduced-motion` setting wins.
 */

export const motion = {
  duration: {
    fast: 'var(--motion-duration-fast)',
    base: 'var(--motion-duration-base)',
    slow: 'var(--motion-duration-slow)',
  },
  easing: {
    out: 'var(--motion-ease-out)',
    'in-out': 'var(--motion-ease-in-out)',
    spring: 'var(--motion-ease-spring)',
  },
} as const;

export type MotionDurationKey = keyof typeof motion.duration;
export type MotionEasingKey = keyof typeof motion.easing;
