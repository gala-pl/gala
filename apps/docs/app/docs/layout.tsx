import type { ReactNode } from 'react';
import { Navbar } from '@/components/navbar';
import { Sidebar } from '@/components/sidebar';
import { source } from '@/lib/source';

export default function Layout({ children }: { children: ReactNode }) {
  const tree = source.getPageTree();

  return (
    <div className="flex flex-col min-h-screen">
      <Navbar tree={tree} />
      <div className="flex flex-1">
        <Sidebar tree={tree} />
        <main className="flex-1 min-w-0">{children}</main>
      </div>
    </div>
  );
}
