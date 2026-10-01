import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TraceIdBadge } from '@/components/common/TraceIdBadge';

describe('TraceIdBadge', () => {
  it('shows truncated id', () => {
    render(<TraceIdBadge traceId="abcdefgh-1234" />);
    expect(screen.getByText(/trace:abcdefgh/)).toBeInTheDocument();
  });

  it('renders nothing when null', () => {
    const { container } = render(<TraceIdBadge traceId={null} />);
    expect(container.firstChild).toBeNull();
  });
});
