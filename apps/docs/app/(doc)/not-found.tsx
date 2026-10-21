import { DocsPage } from "fumadocs-ui/page";

export const metadata = { title: "Page not found | Gala" };

export default function NotFound() {
  return (
    <DocsPage>
      <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
        <h1 className="border-none py-0! text-6xl font-bold text-fd-foreground">404</h1>
        <p className="mt-4 text-lg text-fd-muted-foreground">This page could not be found.</p>
      </div>
    </DocsPage>
  );
}
