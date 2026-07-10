'use client';

import type { ReactNode } from 'react';
import { useEffect, useState } from 'react';
import { cn } from '@/lib/utils';

interface TOCItem {
  title: ReactNode;
  url: string;
  depth: number;
}

export function TOC({ items }: { items: TOCItem[] }) {
  const [activeId, setActiveId] = useState<string>('');

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id);
          }
        }
      },
      { rootMargin: '-80px 0px -80% 0px' }
    );

    for (const item of items) {
      const id = item.url.replace('#', '');
      const el = document.querySelector(`#${id}`);
      if (el) {
        observer.observe(el);
      }
    }

    return () => observer.disconnect();
  }, [items]);

  if (items.length === 0) {
    return null;
  }

  return (
    <aside className="hidden xl:block w-56 shrink-0">
      <div className="sticky top-14 p-4">
        <h3 className="mb-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
          On this page
        </h3>
        <nav className="flex flex-col gap-1">
          {items.map((item) => (
            <a
              key={item.url}
              href={item.url}
              className={cn(
                'text-sm py-1 text-muted-foreground hover:text-foreground transition-colors',
                item.depth > 2 && 'pl-3',
                activeId === item.url.replace('#', '') && 'text-foreground font-medium'
              )}
            >
              {item.title}
            </a>
          ))}
        </nav>
      </div>
    </aside>
  );
}
