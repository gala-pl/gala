# Running Cookbook Recipes

Every recipe is a compilable `.gala` file. Use the `gala` CLI to check and run it.

## Prerequisites

Build the compiler first:

```bash
cargo build --release
```

## Check a recipe (compile only)

```bash
cargo run --release --bin gala -- check cookbook/beginner/hello_world/hello_world.gala
```

## Run a recipe

```bash
cargo run --release --bin gala -- run cookbook/beginner/hello_world/hello_world.gala
```

## Check all recipes

```bash
for f in $(find cookbook -name '*.gala'); do
  cargo run --release --bin gala -- check "$f"
done
```

## Output verification

Some recipes include an `output.golden` file with the expected stdout. Compare:

```bash
cargo run --release --bin gala -- run recipe.gala | diff - output.golden
```
