import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ModeToggle } from '@/components/mode-toggle';
import { ThemeProvider } from '@/components/theme-provider';

function renderWithProviders(ui: React.ReactElement) {
  return render(<ThemeProvider>{ui}</ThemeProvider>);
}

describe('ModeToggle', () => {
  it('renders without crashing', () => {
    renderWithProviders(<ModeToggle />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('has light and dark mode icons', () => {
    renderWithProviders(<ModeToggle />);
    const button = screen.getByRole('button');
    expect(button).toBeInTheDocument();
    expect(button.querySelector('svg')).toBeInTheDocument();
  });
});
