import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, 'src/server.ts'),
      name: 'GalaMCP',
      formats: ['es'],
      fileName: 'server',
    },
    outDir: 'dist',
    rollupOptions: {
      external: [
        'node:process',
        'node:path',
        'node:url',
        'node:child_process',
        'node:fs',
        'node:util',
        'node:events',
        'node:stream',
        'node:buffer',
        '@modelcontextprotocol/sdk',
        'zod',
        'ajv',
        'ajv-formats',
      ],
      output: {
        globals: {},
      },
    },
    target: 'node20',
    platform: 'node',
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
});