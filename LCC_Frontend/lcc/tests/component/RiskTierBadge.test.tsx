import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RiskTierBadge } from '@lcc/approval-gate';

describe('RiskTierBadge', () => {
  it('renders tier label', () => {
    render(<RiskTierBadge tier={3} />);
    expect(screen.getByText(/tier 3/i)).toBeInTheDocument();
  });
});
