import type { Root } from 'fumadocs-core/server';
import Link from 'next/link';
import { ChevronLeft, ChevronRight } from 'lucide-react';

interface PageItem {
  name: string;
  url: string;
}

function flattenPages(tree: Root): PageItem[] {
  const pages: PageItem[] = [];

  function walk(nodes: Node[], parents: string[] = []) {
    for (const node of nodes) {
      if (node.type === 'page') {
        pages.push({ name: node.name as string, url: node.url });
      }
      if (node.type === 'folder') {
        const children = 'children' in node ? node.children : [];
        walk(children as Node[], [...parents, node.name as string]);
      }
    }
  }

  const children = 'children' in tree ? tree.children : [];
  walk(children as Node[]);
  return pages;
}

export function PageNavigation({ tree, currentUrl }: { tree: Root; currentUrl: string }) {
  const pages = flattenPages(tree);
  const currentIndex = pages.findIndex(
    (p) => p.url === currentUrl || p.url === currentUrl + '/'
  );

  if (currentIndex === -1) return null;

  const prev = currentIndex > 0 ? pages[currentIndex - 1] : null;
  const next = currentIndex < pages.length - 1 ? pages[currentIndex + 1] : null;

  if (!prev && !next) return null;

  return (
    <div className="flex items-center justify-between mt-12 pt-6 border-t">
      {prev ? (
        <Link
          href={prev.url}
          className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors group"
        >
          <ChevronLeft className="h-4 w-4 group-hover:-translate-x-0.5 transition-transform" />
          <div className="text-left">
            <div className="text-xs text-muted-foreground">Previous</div>
            <div className="font-medium">{prev.name}</div>
          </div>
        </Link>
      ) : (
        <div />
      )}
      {next ? (
        <Link
          href={next.url}
          className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors group text-right"
        >
          <div>
            <div className="text-xs text-muted-foreground">Next</div>
            <div className="font-medium">{next.name}</div>
          </div>
          <ChevronRight className="h-4 w-4 group-hover:translate-x-0.5 transition-transform" />
        </Link>
      ) : (
        <div />
      )}
    </div>
  );
}
