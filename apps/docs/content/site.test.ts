import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { expect, test } from "vitest";

const requiredPages = [
  "index.mdx",
  "getting-started/index.mdx",
  "getting-started/installation.mdx",
  "getting-started/quickstart.mdx",
  "language/index.mdx",
  "language/syntax.mdx",
  "language/types.mdx",
  "language/effects.mdx",
  "architecture/index.mdx",
  "architecture/pipeline.mdx",
  "architecture/gir.mdx",
  "architecture/backends.mdx",
  "toolchain/cli.mdx",
  "toolchain/diagnostics.mdx",
  "toolchain/formatter-lsp.mdx",
  "toolchain/testing.mdx",
  "standard-library/index.mdx",
  "standard-library/modules.mdx",
  "reference/index.mdx",
  "reference/status.mdx",
  "cookbook/index.mdx",
  "contributing.mdx",
  "roadmap.mdx",
  "resources.mdx",
];

test("Gala documentation contains the public page set", async () => {
  const docsPath = resolve(process.cwd(), "content", "docs");
  const files = await Promise.all(requiredPages.map(async (page) => readFile(resolve(docsPath, page), "utf8")));

  expect(files).toHaveLength(requiredPages.length);
  expect(files.every((file) => file.includes("title:"))).toBe(true);
  expect(files.every((file) => file.includes("description:"))).toBe(true);
});
