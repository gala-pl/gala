import { absoluteUrl, siteConfig } from "@/lib/site";
import { source } from "@/loaders/source";

export const revalidate = false;

function stringifyTitle(title: any): string {
  if (typeof title === "string") return title;
  if (typeof title === "number" || typeof title === "bigint" || typeof title === "boolean") return String(title);
  if (title && typeof title === "object") {
    if (title.props?.children) return stringifyTitle(title.props.children);
    if (Array.isArray(title)) return title.map(stringifyTitle).join("");
    if (title.textContent) return String(title.textContent);
    if (title.props?.children && typeof title.props.children === "string") return title.props.children;
  }
  return "";
}

export function GET() {
  const pages = source.getPages();
  const lines = ["# " + siteConfig.name, "", "> " + siteConfig.description, ""];

  for (const page of pages) {
    const title = stringifyTitle(page.data.title) || "Untitled";
    lines.push("## " + title, "");
    lines.push("- [" + title + "](" + absoluteUrl(page.url) + "): " + (page.data.description || "Documentation page"));

    for (const section of page.data.toc?.filter((item: any) => item.depth >= 2 && item.depth <= 4) ?? []) {
      const sectionTitle = stringifyTitle(section.title);
      if (sectionTitle)
        lines.push("- [" + sectionTitle + "](" + absoluteUrl(page.url + "?id=" + section.url.replace("#", "")) + ")");
    }

    lines.push("");
  }

  lines.push("---", "", "This site contains documentation for the Gala programming language.");

  return new Response(lines.join("\\n"), {
    headers: { "Content-Type": "text/plain; charset=utf-8" },
  });
}
