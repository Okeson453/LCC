/**
 * Custom LinkedIn OAuth provider for NextAuth.
 *
 * The actual OAuth exchange is performed by the backend (`/auth/linkedin/start`,
 * `/auth/linkedin/callback`) — the frontend just redirects. This provider
 * shape is kept minimal and includes a custom profile mapper.
 */

import type { OAuthConfig } from 'next-auth/providers';

interface LinkedInProfile {
  sub: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  email?: string;
  picture?: string;
}

export default function LinkedInProvider(): OAuthConfig<LinkedInProfile> {
  return {
    id: 'linkedin',
    name: 'LinkedIn',
    type: 'oauth',
    wellKnown: 'https://www.linkedin.com/oauth/.well-known/openid-configuration',
    authorization: { params: { scope: 'openid profile email w_member_social' } },
    checks: ['pkce', 'state'],
    clientId: process.env.LINKEDIN_CLIENT_ID ?? '',
    clientSecret: process.env.LINKEDIN_CLIENT_SECRET ?? '',
    profile(profile) {
      return {
        id: profile.sub,
        name: profile.name,
        email: profile.email,
        image: profile.picture,
      };
    },
  };
}
