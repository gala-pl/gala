'use client';

import type { Root } from 'fumadocs-core/page-tree';
import { GitFork, Menu } from 'lucide-react';
import Link from 'next/link';
import { ModeToggle } from '@/components/mode-toggle';
import { SearchDialog } from '@/components/search-dialog';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet';

const GITHUB_URL = 'https://github.com/anomalyco/gala';

function SidebarItems({ tree }: { tree: Root }) {
  const children = 'children' in tree ? tree.children : [];

  return (
    <div className="flex flex-col gap-1">
      {children.map((node) => {
        if (node.type === 'separator') {
          return (
            <div
              key={String(node.name)}
              className="px-3 py-1 text-xs font-medium text-muted-foreground"
            >
              {node.name}
            </div>
          );
        }
        if (node.type === 'page') {
          return (
            <Link
              key={node.url}
              href={node.url}
              className="px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-accent rounded-md transition-colors"
            >
              {node.name}
            </Link>
          );
        }
        if (node.type === 'folder') {
          return (
            <div key={String(node.name)}>
              <div className="px-3 py-1 text-xs font-semibold text-foreground">{node.name}</div>
              <div className="ml-2 flex flex-col gap-0.5">
                {'children' in node &&
                  node.children.map((child) => {
                    if (child.type === 'page') {
                      return (
                        <Link
                          key={child.url}
                          href={child.url}
                          className="px-3 py-1 text-sm text-muted-foreground hover:text-foreground hover:bg-accent rounded-md transition-colors"
                        >
                          {child.name}
                        </Link>
                      );
                    }
                    return null;
                  })}
              </div>
            </div>
          );
        }
        return null;
      })}
    </div>
  );
}

export function Navbar({ tree }: { tree: Root }) {
  return (
    <header className="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="flex h-14 items-center px-4 lg:px-6">
        <Sheet>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className="md:hidden mr-2">
              <Menu className="h-5 w-5" />
              <span className="sr-only">Open menu</span>
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className="w-72 p-0">
            <ScrollArea className="flex-1 p-4">
              <SidebarItems tree={tree} />
            </ScrollArea>
          </SheetContent>
        </Sheet>

        <Link href="/" className="font-bold text-lg mr-6">
          Gala
        </Link>

        <nav className="hidden md:flex items-center gap-1 text-sm mr-auto">
          <Link
            href="/docs"
            className="px-3 py-1.5 text-muted-foreground hover:text-foreground hover:bg-accent rounded-md transition-colors"
          >
            Docs
          </Link>
        </nav>

        <div className="flex items-center gap-1 ml-auto">
          <SearchDialog />
          <ModeToggle />
          <Button variant="ghost" size="icon" asChild>
            <a href={GITHUB_URL} target="_blank" rel="noreferrer">
              <GitFork className="h-5 w-5" />
              <span className="sr-only">GitHub</span>
            </a>
          </Button>
        </div>
      </div>
    </header>
  );
}
