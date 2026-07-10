import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Sidebar } from '@/components/sidebar';

vi.mock('next/navigation', () => ({
  usePathname: () => '/docs/language-spec',
}));

const mockTree = {
  name: 'root',
  children: [
    { type: 'page', name: 'Language Spec', url: '/docs/language-spec' },
    { type: 'page', name: 'Guides', url: '/docs/guides' },
    {
      type: 'folder',
      name: 'Advanced',
      children: [
        { type: 'page', name: 'Type System', url: '/docs/language-spec/type-system' },
      ],
    },
  ],
};

describe('Sidebar', () => {
  it('renders page links', () => {
    render(<Sidebar tree={mockTree} />);
    expect(screen.getByText('Language Spec')).toBeInTheDocument();
    expect(screen.getByText('Guides')).toBeInTheDocument();
  });

  it('renders folder names', () => {
    render(<Sidebar tree={mockTree} />);
    expect(screen.getByText('Advanced')).toBeInTheDocument();
  });

  it('renders child pages inside folders', () => {
    render(<Sidebar tree={mockTree} />);
    expect(screen.getByText('Type System')).toBeInTheDocument();
  });
});