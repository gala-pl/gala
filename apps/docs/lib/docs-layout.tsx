import { baseOptions } from "@/app/layout.config";
import { SidebarItem, SidebarSeparator } from "@/components/sidebar-item";
import { source } from "@/loaders/source";
import type { DocsLayoutProps } from "fumadocs-ui/layouts/docs";

export const layoutProps: DocsLayoutProps = {
  ...baseOptions,
  tree: source.pageTree,
  sidebar: {
    className: "docs-sidebar",
    hideSearch: false,
    collapsible: false,
    prefetch: false,
    components: {
      Item: SidebarItem,
      Separator: SidebarSeparator,
    },
  },
  nav: {
    ...baseOptions.nav,
    transparentMode: "top",
    enabled: true,
    enableSearch: true,
  },
};
