import { CopyMarkdownButton } from "@/components/copy-markdown-button";
import { Heading } from "@/components/heading";
import { Tabs } from "@/components/tabs";
import { getLLMText } from "@/loaders/get-llm-text";
import { source } from "@/loaders/source";
import { absoluteUrl, siteConfig } from "@/lib/site";
import { Callout } from "fumadocs-ui/components/callout";
import { findNeighbour } from "fumadocs-core/server";
import defaultMdxComponents, { createRelativeLink } from "fumadocs-ui/mdx";
import { DocsBody, DocsPage, DocsTitle } from "fumadocs-ui/page";
import { Github } from "lucide-react";
import { notFound } from "next/navigation";

export const revalidate = false;
export const dynamic = "force-static";

export default async function Page(props: { params: Promise<{ slug?: string[] }> }) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  const markdownContent = await getLLMText(page);
  const MDXContent = page.data.body;
  const editUrl = siteConfig.repositoryUrl
    ? siteConfig.repositoryUrl + "/edit/main/content/docs/" + page.file.path
    : undefined;

  return (
    <DocsPage
      toc={page.data.toc.filter((item) => item.depth <= 3)}
      tableOfContent={{ style: "clerk", single: false }}
      full={false}
      footer={{ items: findNeighbour(source.pageTree, page.url) }}
    >
      {page.data.title && (
        <div className="mb-6">
          <DocsTitle>{page.data.title}</DocsTitle>
          <div className="h-2" />
          <div className="flex items-center gap-2">
            <CopyMarkdownButton content={markdownContent} />
            {editUrl ? (
              <a
                href={editUrl}
                target="_blank"
                rel="noreferrer noopener"
                className="inline-flex items-center gap-1.5 rounded-md border border-[var(--color-fd-border)] px-2 py-1 text-xs text-fd-muted-foreground transition-colors hover:bg-fd-muted/50 hover:text-fd-foreground"
              >
                <Github className="h-3 w-3" />
                Edit this page
              </a>
            ) : null}
          </div>
        </div>
      )}
      <DocsBody>
        <MDXContent
          components={{
            ...defaultMdxComponents,
            a: createRelativeLink(source, page),
            blockquote: Callout,
            Tabs,
            h1: (props) => <Heading as="h1" {...props} />,
            h2: (props) => <Heading as="h2" {...props} />,
            h3: (props) => <Heading as="h3" {...props} />,
            h4: (props) => <Heading as="h4" {...props} />,
            h5: (props) => <Heading as="h5" {...props} />,
            h6: (props) => <Heading as="h6" {...props} />,
          }}
        />
      </DocsBody>
    </DocsPage>
  );
}

export function generateStaticParams() {
  return source.generateParams();
}

export async function generateMetadata(props: { params: Promise<{ slug?: string[] }> }) {
  const params = await props.params;
  const page = source.getPage(params.slug);
  if (!page) notFound();

  const title = (page.data.title ?? "Documentation") + " | " + siteConfig.name;
  const description = page.data.description ?? siteConfig.description;
  const url = absoluteUrl(page.url);

  return {
    title,
    description,
    alternates: { canonical: url },
    openGraph: {
      type: "website",
      title,
      description,
      siteName: siteConfig.name,
      url,
      images: [
        {
          url: absoluteUrl("/og.png?title=" + encodeURIComponent(page.data.title ?? "Documentation")),
          width: 1200,
          height: 630,
          alt: title,
        },
      ],
    },
    twitter: { card: "summary_large_image", title, description },
    keywords: ["Gala", "programming language", "documentation"],
  };
}
