import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StaleContactBanner } from '@/components/crm';

describe('StaleContactBanner', () => {
  it('renders when count > 0', () => {
    render(<StaleContactBanner count={3} />);
    expect(screen.getByText(/3 contacts are stale/i)).toBeInTheDocument();
  });

  it('renders nothing when count <= 0', () => {
    const { container } = render(<StaleContactBanner count={0} />);
    expect(container.firstChild).toBeNull();
  });
});
