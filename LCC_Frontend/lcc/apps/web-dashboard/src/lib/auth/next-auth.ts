/**
 * NextAuth.js config — LinkedIn OAuth provider.
 *
 * The session callback reads the backend's JWT (issued by /auth/linkedin/callback)
 * from the cookie and exposes it as `session.accessToken` for the API client.
 */

import type { NextAuthOptions } from 'next-auth';
import LinkedInProvider from './providers/linkedin';

export const authOptions: NextAuthOptions = {
  providers: [LinkedInProvider()],
  session: { strategy: 'jwt', maxAge: 30 * 24 * 60 * 60 },
  pages: {
    signIn: '/auth/linkedin/start',
    error: '/auth/error',
  },
  callbacks: {
    async jwt({ token, account }) {
      if (account?.access_token) {
        token.accessToken = account.access_token;
        token.refreshToken = account.refresh_token;
      }
      return token;
    },
    async session({ session, token }) {
      (session as Record<string, unknown>).accessToken = token.accessToken;
      (session as Record<string, unknown>).refreshToken = token.refreshToken;
      return session;
    },
  },
};
