/**
 * NEXT_PUBLIC_* types.
 */

declare namespace NodeJS {
  interface ProcessEnv {
    NEXT_PUBLIC_API_BASE?: string;
    NEXT_PUBLIC_WS_BASE?: string;
    NEXT_PUBLIC_ENVIRONMENT?: 'development' | 'staging' | 'production';
    NEXT_PUBLIC_SENTRY_DSN?: string;
    NEXT_PUBLIC_POSTHOG_KEY?: string;
    NEXT_PUBLIC_POSTHOG_HOST?: string;
    LINKEDIN_CLIENT_ID?: string;
    LINKEDIN_CLIENT_SECRET?: string;
    NEXTAUTH_SECRET?: string;
    NEXTAUTH_URL?: string;
    SENTRY_AUTH_TOKEN?: string;
  }
}
