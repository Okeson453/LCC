/**
 * Typed env loader — only NEXT_PUBLIC_* allowed.
 */

interface PublicEnv {
  NEXT_PUBLIC_API_BASE: string;
  NEXT_PUBLIC_WS_BASE: string;
  NEXT_PUBLIC_ENVIRONMENT: 'development' | 'staging' | 'production';
  NEXT_PUBLIC_SENTRY_DSN?: string;
  NEXT_PUBLIC_POSTHOG_KEY?: string;
}

export const publicEnv: PublicEnv = {
  NEXT_PUBLIC_API_BASE: process.env.NEXT_PUBLIC_API_BASE ?? '',
  NEXT_PUBLIC_WS_BASE: process.env.NEXT_PUBLIC_WS_BASE ?? '',
  NEXT_PUBLIC_ENVIRONMENT:
    (process.env.NEXT_PUBLIC_ENVIRONMENT as PublicEnv['NEXT_PUBLIC_ENVIRONMENT']) ?? 'development',
  NEXT_PUBLIC_SENTRY_DSN: process.env.NEXT_PUBLIC_SENTRY_DSN,
  NEXT_PUBLIC_POSTHOG_KEY: process.env.NEXT_PUBLIC_POSTHOG_KEY,
};

export function isProduction(): boolean {
  return publicEnv.NEXT_PUBLIC_ENVIRONMENT === 'production';
}

/**
 * Server-side environment accessor.
 * Throws if called in the browser so secrets never leak client-side.
 * Client code should read typed values from `publicEnv` instead.
 */
export function env(name: string): string {
  if (typeof window !== 'undefined') {
    throw new Error('[env] "'+name+'" was accessed from the client; use publicEnv for client values.');
  }
  const value = process.env[name];
  if (value === undefined) {
    throw new Error('[env] "'+name+'" is not defined on the server.');
  }
  return value;
}

