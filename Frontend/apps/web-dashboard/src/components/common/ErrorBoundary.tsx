'use client';

import * as React from 'react';
import { ErrorState, Button } from '@lcc/ui';

interface Props {
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  override componentDidCatch(error: Error, info: React.ErrorInfo): void {
    if (typeof window !== 'undefined' && (window as unknown as { Sentry?: unknown }).Sentry) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).Sentry?.captureException?.(error, { extra: info });
    }
  }

  reset = (): void => this.setState({ error: null });

  override render(): React.ReactNode {
    if (this.state.error) {
      return this.props.fallback ?? (
        <ErrorState
          title="Component crashed"
          description={this.state.error.message}
          onRetry={this.reset}
        />
      );
    }
    return this.props.children;
  }
}
