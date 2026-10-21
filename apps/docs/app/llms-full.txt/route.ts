import { siteConfig } from "@/lib/site";
import { getLLMText } from "@/loaders/get-llm-text";
import { source } from "@/loaders/source";

export const revalidate = false;

export async function GET() {
  const pages = source.getPages();
  const sections = ["# " + siteConfig.name, "", siteConfig.description, ""];

  for (const page of pages) {
    sections.push(await getLLMText(page), "");
  }

  return new Response(sections.join("\\n"), {
    headers: { "Content-Type": "text/plain; charset=utf-8" },
  });
}
