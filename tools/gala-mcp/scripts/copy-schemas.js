import { copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const srcDir = resolve(__dirname, '../../../schemas/v1');
const destDir = resolve(__dirname, '../dist');

if (!existsSync(destDir)) {
  mkdirSync(destDir, { recursive: true });
}

const files = ['diagnostic.json', 'gir.json', 'property_result.json'];

for (const file of files) {
  const src = join(srcDir, file);
  const dest = join(destDir, file);
  copyFileSync(src, dest);
}
