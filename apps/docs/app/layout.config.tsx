import { siteConfig } from "@/lib/site";
import type { BaseLayoutProps } from "fumadocs-ui/layouts/shared";
import { Code2 } from "lucide-react";

export const logo = (
  <span className="inline-flex items-center gap-2 font-semibold tracking-tight">
    <Code2 aria-hidden="true" className="size-5" />
    <span>{siteConfig.name}</span>
  </span>
);

export const baseOptions: BaseLayoutProps = {
  nav: {
    title: logo,
  },
  links: [
    {
      text: "Documentation",
      url: "/docs",
      active: "nested-url",
    },
  ],
};
