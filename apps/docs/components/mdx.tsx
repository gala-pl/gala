import type { MDXComponents } from 'mdx/types';
import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

function Callout({
  title,
  type,
  children,
}: {
  title?: string;
  type?: string;
  children: ReactNode;
}) {
  return (
    <div
      className={cn(
        'my-4 rounded-lg border p-4',
        type === 'warn' && 'border-amber-500/50 bg-amber-50 dark:bg-amber-950/20',
        type === 'error' && 'border-red-500/50 bg-red-50 dark:bg-red-950/20',
        (!type || type === 'info') && 'border-blue-500/50 bg-blue-50 dark:bg-blue-950/20'
      )}
    >
      {title && <div className="font-semibold mb-1 text-sm">{title}</div>}
      <div className="text-sm [&>p]:my-1">{children}</div>
    </div>
  );
}

function Card({
  title,
  description,
  href,
  icon,
}: {
  title?: string;
  description?: string;
  href?: string;
  icon?: ReactNode;
}) {
  const Comp = href ? 'a' : 'div';
  return (
    <Comp href={href} className="block rounded-lg border p-4 hover:bg-accent transition-colors">
      {icon && <div className="mb-2">{icon}</div>}
      {title && <div className="font-semibold text-sm">{title}</div>}
      {description && <div className="text-xs text-muted-foreground mt-1">{description}</div>}
    </Comp>
  );
}

function Cards({ children }: { children: ReactNode }) {
  return <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 my-4">{children}</div>;
}

export function getMDXComponents(components?: MDXComponents): MDXComponents {
  return {
    h1: ({ children, ...props }) => (
      <h1 className="scroll-m-20 text-4xl font-bold tracking-tight" {...props}>
        {children}
      </h1>
    ),
    h2: ({ children, ...props }) => (
      <h2 className="scroll-m-20 text-2xl font-semibold tracking-tight mt-10 mb-4" {...props}>
        {children}
      </h2>
    ),
    h3: ({ children, ...props }) => (
      <h3 className="scroll-m-20 text-xl font-semibold tracking-tight mt-8 mb-3" {...props}>
        {children}
      </h3>
    ),
    h4: ({ children, ...props }) => (
      <h4 className="scroll-m-20 text-lg font-semibold mt-6 mb-2" {...props}>
        {children}
      </h4>
    ),
    p: ({ children, ...props }) => (
      <p className="leading-7 [&:not(:first-child)]:mt-4" {...props}>
        {children}
      </p>
    ),
    a: ({ children, href, ...props }) => (
      <a
        href={href}
        className="text-primary underline underline-offset-4 hover:text-primary/80"
        {...props}
      >
        {children}
      </a>
    ),
    ul: ({ children, ...props }) => (
      <ul className="my-4 ml-6 list-disc [&>li]:mt-2" {...props}>
        {children}
      </ul>
    ),
    ol: ({ children, ...props }) => (
      <ol className="my-4 ml-6 list-decimal [&>li]:mt-2" {...props}>
        {children}
      </ol>
    ),
    code: ({ children, ...props }) => (
      <code
        className="relative rounded bg-muted px-[0.3rem] py-[0.2rem] text-sm font-mono"
        {...props}
      >
        {children}
      </code>
    ),
    pre: ({ children, ...props }) => (
      <pre
        className="my-4 overflow-x-auto rounded-lg border bg-muted p-4 text-sm font-mono"
        {...props}
      >
        {children}
      </pre>
    ),
    blockquote: ({ children, ...props }) => (
      <blockquote className="mt-4 border-l-4 border-primary pl-4 italic" {...props}>
        {children}
      </blockquote>
    ),
    table: ({ children, ...props }) => (
      <div className="my-4 overflow-x-auto">
        <table className="w-full border-collapse text-sm" {...props}>
          {children}
        </table>
      </div>
    ),
    th: ({ children, ...props }) => (
      <th className="border-b px-4 py-2 text-left font-medium" {...props}>
        {children}
      </th>
    ),
    td: ({ children, ...props }) => (
      <td className="border-b px-4 py-2" {...props}>
        {children}
      </td>
    ),
    hr: (props) => <hr className="my-6 border-t" {...props} />,
    img: ({ alt, ...props }) => (
      // eslint-disable-next-line @next/next/no-img-element
      <img alt={alt ?? ''} className="rounded-lg my-4" {...props} />
    ),
    Callout,
    Card,
    Cards,
    ...components,
  };
}

export const useMDXComponents = getMDXComponents;

declare global {
  type MDXProvidedComponents = ReturnType<typeof getMDXComponents>;
}
