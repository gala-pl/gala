---
paths:
  - "**/*.rs"
---

# Rust Conventions

## Style
- Follow rustfmt with max_width = 100, use_small_heuristics = "Max"
- Reorder imports and modules (enforced by rustfmt)
- Use `camelCase` for type names, `snake_case` for functions/variables, `SCREAMING_SNAKE_CASE` for constants
- Prefer `Self` over repeating the type name in impl blocks

## Error Handling
- Errors are values — use `Result<T, E>` and custom diagnostic types
- No `unwrap()`, `expect()`, or `panic!()` on user-reachable paths
- Use `anyhow` for binary code, custom error enums for library code
- Propagate errors with `?` operator; convert with `.map_err()` or `Into`
- Internal invariants asserted only in debug builds with `debug_assert!`

## Documentation
- All public API items must have `///` doc comments
- Include examples in doc comments where useful
- Mark internal implementation details with `//` comments, not doc comments
- Use `// SAFETY:` comments for unsafe blocks explaining the safety invariant

## Performance
- Avoid unnecessary allocations; prefer slices and iterators
- Use `Box<[T]>` over `Vec<T>` for frozen collections
- Profile before optimizing; don't add complexity without measurement
