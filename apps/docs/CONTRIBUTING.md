# Contributing to Gala documentation

## Development

Install Nub 0.8.3, then install dependencies and start the site:

    nub install
    nub run dev

The development server runs at localhost:3000.

## Content

Documentation pages live in content/docs. Keep frontmatter titles and descriptions clear, use relative links between pages, and avoid documenting language behavior that is not established by the Gala specification.

## Validation

Before opening a change, run:

    nub run test
    nub run format:check
    nub run lint:check
    nub run build

## Pull requests

Keep changes focused, explain user-visible behavior, and include screenshots for substantial visual changes.
