'use client';

import { getBreadcrumbItems } from 'fumadocs-core/breadcrumb';
import Link from 'next/link';
import { cn } from '@/lib/utils';

interface BreadcrumbItem {
  name: string;
  url: string;
}

export function Breadcrumbs() {
  const path = window.location.pathname || '/docs';
  const url = path === '/' ? '/docs' : path;
  const items = getBreadcrumbItems(url, { includePage: true });
  
  // Process items into simplified structure
  const breadcrumbItems = items.map(item => {
    return {
      label: (typeof item.name === 'string' ? item.name : item.name.toString()) || '',
      href: item.url || '/'
    };
  }).filter(item => item.label);
  
  if (breadcrumbItems.length <= 1) return null;
  
  return (
    <nav className="flex items-center gap-1 text-sm text-muted-foreground mb-6">
      <Link href="/docs" className="text-sm text-muted-foreground hover:text-foreground transition-colors">Docs</Link>
      {breadcrumbItems.map((crumb, i) => (
        <span key={crumb.href} className="flex items-center gap-1">
          <span>/</span>
          {i === breadcrumbItems.length - 1 ? (
            <span className="text-foreground font-medium">{crumb.label}</span>
          ) : (
            <Link href={crumb.href} className="hover:text-foreground transition-colors">
              {crumb.label}
            </Link>
          )}
        </span>
      ))}
    </nav>
  );
}