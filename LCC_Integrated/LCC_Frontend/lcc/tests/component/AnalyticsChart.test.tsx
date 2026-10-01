import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AnalyticsChart } from '@/components/analytics';

describe('AnalyticsChart', () => {
  it('renders title', () => {
    render(<AnalyticsChart title="Reach" series={[{ x: 'd1', y: 1 }, { x: 'd2', y: 3 }]} />);
    expect(screen.getByText(/Reach/)).toBeInTheDocument();
  });
});
