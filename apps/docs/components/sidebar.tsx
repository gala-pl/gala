import type { Node, Root } from 'fumadocs-core/page-tree';
import Link from 'next/link';
import { ScrollArea } from '@/components/ui/scroll-area';

function TreeNode({ node }: { node: Node }) {
  if (node.type === 'separator') {
    return <div className="px-3 py-1 text-xs font-medium text-muted-foreground">{node.name}</div>;
  }

  if (node.type === 'page') {
    return (
      <Link
        href={node.url}
        className="block px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-accent rounded-md transition-colors"
      >
        {node.name}
      </Link>
    );
  }

  if (node.type === 'folder') {
    return (
      <div className="mb-2">
        <div className="px-3 py-1 text-xs font-semibold text-foreground uppercase tracking-wider">
          {node.name}
        </div>
        <div className="ml-2 flex flex-col gap-0.5">
          {'children' in node &&
            node.children.map((child, idx) => (
              <TreeNode
                key={'url' in child && typeof child.url === 'string' ? child.url : `node-${idx}`}
                node={child}
              />
            ))}
        </div>
      </div>
    );
  }

  return null;
}

export function Sidebar({ tree }: { tree: Root }) {
  const children = 'children' in tree ? tree.children : [];

  return (
    <aside className="hidden md:block w-64 shrink-0 border-r">
      <ScrollArea className="h-[calc(100vh-3.5rem)] p-4">
        <div className="flex flex-col gap-1">
          {children.map((node, idx) => (
            <TreeNode
              key={'url' in node && typeof node.url === 'string' ? node.url : `tree-${idx}`}
              node={node}
            />
          ))}
        </div>
      </ScrollArea>
    </aside>
  );
}
