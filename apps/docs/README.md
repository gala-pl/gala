# @gala/docs

Documentation website for the Gala programming language, built with [Next.js](https://nextjs.org) 16 and [Fumadocs](https://fumadocs.vercel.app).

## Tech Stack

| Tool | Purpose |
|------|---------|
| **Next.js 16** | React framework with Turbopack |
| **Fumadocs** | MDX-based documentation framework |
| **React 19** | UI library |
| **Tailwind CSS 4** | Utility-first styling |
| **shadcn/ui** | Accessible UI primitives (Radix) |
| **TypeScript 5.7** | Strict-mode type safety |
| **Bun** | Package manager and runtime |
| **oxlint** | Fast Rust-based linter |
| **ultracite** | Opinionated formatting & linting |

## Getting Started

```bash
# Install dependencies
cd apps/docs && bun install

# Start dev server
bun next dev

# Build for production
bun next build

# Typecheck
bun tsc --noEmit

# Lint
bunx oxlint
```

The dev server runs at `http://localhost:3000`.

## Project Structure

```
apps/docs/
├── app/
│   ├── page.tsx              # Landing page
│   ├── layout.tsx            # Root layout (theme provider)
│   ├── docs/
│   │   ├── layout.tsx        # Docs layout (navbar + sidebar + content)
│   │   └── [[...slug]]/
│   │       └── page.tsx      # Catch-all docs page (MDX rendering)
│   └── api/search/
│       └── route.ts          # Full-text search API
├── components/
│   ├── navbar.tsx            # Top navigation bar
│   ├── sidebar.tsx           # Sidebar page tree
│   ├── toc.tsx               # Table of contents (scroll-spy)
│   ├── search-dialog.tsx     # Command palette search (Cmd+K)
│   ├── mode-toggle.tsx       # Theme switcher
│   ├── theme-provider.tsx    # next-themes wrapper
│   ├── mdx.tsx               # Custom MDX components (Callout, Card, etc.)
│   └── ui/                   # shadcn/ui primitives
├── content/docs/             # MDX documentation source
│   ├── index.mdx             # Welcome page
│   ├── vision.mdx            # Vision & principles
│   ├── roadmap.mdx           # Development roadmap
│   ├── contributing.mdx      # Contributor guide
│   ├── getting-started/      # Installation, quickstart
│   ├── language-spec/        # Language specification
│   ├── architecture/         # Architecture documentation
│   └── guides/               # Standard library, testing, toolchain
├── source.config.ts          # Fumadocs source configuration
├── next.config.mjs           # Next.js configuration (with MDX)
├── tailwind.config.ts        # Tailwind theme
└── tsconfig.json             # TypeScript configuration
```

## Content

Documentation is authored in MDX and stored in `content/docs/`. The content covers:

- **Language specification** — type system, syntax, semantics
- **Architecture** — compiler pipeline, crate architecture, backend design
- **Getting started** — installation, quickstart tutorial
- **Guides** — standard library, testing, toolchain, debugging
- **Vision & roadmap** — project goals, milestones, design principles

### Adding Documentation

Create a new `.mdx` file in the appropriate subdirectory under `content/docs/`. The file is automatically picked up by Fumadocs and added to the page tree and search index.

Frontmatter example:

```mdx
---
title: My New Page
description: A brief description for search and navigation
---

Content here...
```

## Scripts

| Script | Description |
|--------|-------------|
| `bun dev` | Start dev server with Turbopack |
| `bun build` | Production build |
| `bun start` | Start production server |
| `bun tsc --noEmit` | TypeScript type checking |
| `bunx oxlint` | Lint with oxlint |
| `ultracite check` | Run ultracite checks |
| `ultracite fix` | Fix ultracite issues |

## Dependencies

**Runtime:** `next`, `react`, `fumadocs-core`, `fumadocs-mdx`, `next-themes`, `lucide-react`, `cmdk`, `tailwind-merge`, `class-variance-authority`, `clsx`, Radix UI primitives

**Dev:** `typescript`, `tailwindcss`, `oxlint`, `oxfmt`, `ultracite`, `@types/*`
