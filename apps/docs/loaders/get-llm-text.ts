import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { InferPageType } from "fumadocs-core/source";
import { remarkInclude } from "fumadocs-mdx/config";
import matter from "gray-matter";
import { remark } from "remark";
import remarkGfm from "remark-gfm";
import remarkMdx from "remark-mdx";
import remarkStringify from "remark-stringify";

import type { source } from "@/loaders/source";

const processor = remark().use(remarkMdx).use(remarkInclude).use(remarkGfm).use(remarkStringify);

export async function getLLMText(page: InferPageType<typeof source>): Promise<string> {
  const relativePath = page.file.path;
  const resolvedPath = path.join(process.cwd(), "content", "docs", relativePath);

  let filePath: string;
  let fileContent: Buffer;
  try {
    filePath = resolvedPath;
    fileContent = await fs.readFile(filePath);
  } catch {
    filePath = page.data._file.absolutePath;
    fileContent = await fs.readFile(filePath);
  }

  const { content } = matter(fileContent.toString());
  const processed = await processor.process({ path: filePath, value: content });

  return "# " + page.data.title + "\\n\\n" + processed;
}
