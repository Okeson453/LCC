'use client';

/**
 * AuthProvider — mock NextAuth session.
 */

import * as React from 'react';

export interface MockSession {
  memberId: string;
  email: string;
  displayName: string;
  goalMode: 'job_hunting' | 'client_acquisition' | 'hybrid';
  isRestricted: boolean;
  /** Mock JWT — never use a real one in tests. */
  accessToken: string;
}

export interface AuthContextValue {
  session: MockSession | null;
  setSession: (session: MockSession | null) => void;
}

const AuthContext = React.createContext<AuthContextValue | null>(null);

export interface AuthProviderProps {
  children: React.ReactNode;
  initialSession?: MockSession | null;
}

export function AuthProvider({
  children,
  initialSession = null,
}: AuthProviderProps): React.ReactElement {
  const [session, setSession] = React.useState<MockSession | null>(initialSession);
  const value = React.useMemo(() => ({ session, setSession }), [session]);
  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useMockAuth(): AuthContextValue {
  const ctx = React.useContext(AuthContext);
  if (!ctx) throw new Error('useMockAuth must be used within AuthProvider');
  return ctx;
}

export const AuthContextKey = AuthContext;
