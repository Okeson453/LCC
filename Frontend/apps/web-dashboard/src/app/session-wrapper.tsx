/**
 * Client boundary for NextAuth's SessionProvider.
 *
 * next-auth v4 ships `react/index.js` as CJS without a 'use client'
 * directive, so importing SessionProvider directly from a server layout
 * makes Next treat it as a server module and it throws at render
 * ("React Context is unavailable in Server Components"). Re-exporting it
 * from a file that IS marked 'use client' forces the client boundary.
 */

'use client';

export { SessionProvider } from 'next-auth/react';
