# AGENTS.md

This repository contains the Gala documentation site.

## Development commands

Use Nub for dependency management, scripts, and TypeScript execution:

- nub install installs dependencies.
- nub run dev starts the Next.js development server.
- nub run build creates the production build.
- nub run test runs Vitest.
- nub run format:check and nub run lint:check validate formatting and linting.

Nub 0.8.3 and Node.js 24+ are required.

## Rules

- Keep all tests in TypeScript.
- Every behavior change needs focused test coverage.
- Do not add console logging to production code or tests.
- Keep comments short and lowercase when a comment is necessary.
- Preserve the docs-only repository boundary; do not add runtime language implementation code here.
- Keep site identity values centralized in lib/site.ts.
- Keep documentation content in content/docs.
- Use the existing Next.js, Fumadocs, Tailwind, and Vercel setup unless the task explicitly changes the platform.
