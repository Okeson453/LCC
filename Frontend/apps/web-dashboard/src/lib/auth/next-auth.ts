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
      // `Session` is a closed interface, so a direct cast to
      // `Record<string, unknown>` is rejected. Widen once through `unknown`.
      const s = session as unknown as Record<string, unknown>;
      s.accessToken = token.accessToken;
      s.refreshToken = token.refreshToken;
      return session;
    },
  },
};
