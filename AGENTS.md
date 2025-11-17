# Repository Guidelines

## Project Structure & Module Organization
- `extension.toml` defines metadata required by Zed; keep version, id, and description current.
- `Cargo.toml` + `src/lib.rs` implement the Rust WebAssembly module that powers the Git visualization logic; organize complex logic into `src/graph/*` modules before adding UI layers.
- `PLAN.md` tracks milestones, while `/assets` (create when needed) should contain screenshots or icons referenced by README.

## Build, Test, and Development Commands
- `cargo build --target wasm32-wasi` compiles the extension to WebAssembly for Zed to load.
- `cargo test` executes unit tests for parsing, command invocation, and helpers.
- `zed --foreground` runs the editor with verbose logs so you can inspect extension output.

## Coding Style & Naming Conventions
- Rust code uses `rustfmt` defaults (4-space indent, snake_case for functions, UpperCamelCase for types).
- Prefer explicit modules (e.g., `mod parser; mod renderer;`) over large files.
- Files referencing git concepts should be named after their primary concern (`commit_graph.rs`, `branch_colors.rs`) for quick navigation.

## Testing Guidelines
- Unit tests belong alongside the module under `#[cfg(test)]`; integration tests live in `tests/` once APIs stabilize.
- Snapshot CLI output via fixtures that mirror `git log --graph` snippets so regressions are easy to spot.
- Run `cargo test --target wasm32-wasi` before submitting to ensure WASI-specific differences are caught.

## Commit & Pull Request Guidelines
- Commits follow the imperative style: `Add graph parser`, `Fix branch color calculation`.
- Each PR should describe the user-facing change, list testing commands/results, and include screenshots or GIFs for UI tweaks.
- Link issues using `Fixes #NN` when closing roadmap items, and request review from the role owner listed in `PLAN.md`.
