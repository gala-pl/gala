import { layoutProps } from "@/lib/docs-layout";
import { DocsLayout } from "fumadocs-ui/layouts/docs";
import type { ReactNode } from "react";

export const dynamic = "force-static";

export default function Layout({ children }: { children: ReactNode }) {
  return <DocsLayout {...layoutProps}>{children}</DocsLayout>;
}
