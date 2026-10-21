import { Analytics } from "@vercel/analytics/react";
import { RootProvider } from "fumadocs-ui/provider";
import { type ReactNode, Suspense } from "react";

import Scroller from "@/components/scroller";
import { siteConfig } from "@/lib/site";

import "./global.css";

export const metadata = {
  title: {
    default: siteConfig.name,
    template: "%s | Gala",
  },
  description: siteConfig.description,
};

export default function Layout({ children }: { children: ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="flex min-h-screen flex-col">
        <Analytics />
        <RootProvider search={{ enabled: true }} theme={{}}>
          {children}
        </RootProvider>
        <Suspense fallback={null}>
          <Scroller />
        </Suspense>
      </body>
    </html>
  );
}
