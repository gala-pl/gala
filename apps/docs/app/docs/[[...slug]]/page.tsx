import type { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { getMDXComponents } from '@/components/mdx';
import { TOC } from '@/components/toc';
import { source } from '@/lib/source';

interface PageProps {
  params: Promise<{ slug?: string[] }>;
}

export default async function Page(props: PageProps) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) {
    notFound();
  }

  const MDX = page.data.body;

  return (
    <div className="flex">
      <article className="flex-1 min-w-0 px-6 py-8 lg:px-10 max-w-3xl mx-auto">
        <h1 className="scroll-m-20 text-4xl font-bold tracking-tight">{page.data.title}</h1>
        {page.data.description && (
          <p className="text-lg text-muted-foreground mt-2 mb-6">{page.data.description}</p>
        )}
        <div className="prose-custom">
          <MDX components={getMDXComponents()} />
        </div>
      </article>
      <TOC items={page.data.toc} />
    </div>
  );
}

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: PageProps): Promise<Metadata> {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) {
    notFound();
  }

  return {
    description: page.data.description,
    title: page.data.title,
  };
}
