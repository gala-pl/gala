# Contributing a Recipe

## Recipe format

Each recipe is a directory with two required files and one optional file:

```
cookbook/<section>/<recipe_name>/
├── recipe.md              # Walkthrough (required)
├── <recipe_name>.gala     # Runnable source (required)
└── output.golden          # Expected stdout (optional, for CI snapshots)
```

### `.gala` file conventions

- Start with a `///` doc comment describing what this recipe demonstrates.
- Place imports at the top, helper functions next, and `fn main()` last.
- Use `print()` for observable output.
- Must compile with zero diagnostics via `gala check`.
- Follow the existing Gala code style.

### `recipe.md` conventions

- Title: h1 matching the recipe name.
- Sections (in order):
  1. **Goal** — what you'll accomplish.
  2. **Concepts** — Gala features used (link to `docs/` spec pages).
  3. **Code walkthrough** — explain each section of the `.gala` file.
  4. **Expected output** — what `gala run` prints.
  5. **See also** — links to related recipes and docs.

## Recipe checklist

- [ ] `.gala` file compiles with `gala check --no-diagnostics`
- [ ] `.gala` file follows existing Gala style (indentation, naming, effect annotations)
- [ ] `recipe.md` covers goal, concepts, walkthrough, expected output, and cross-references
- [ ] `output.golden` matches actual stdout (if included)
- [ ] Cross-references to `docs/` spec pages resolve correctly
