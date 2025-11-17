# Git Graph (Zed Extension)

Git Graph is a work-in-progress Zed extension that aims to bring an interactive Git history visualization to the editor. The goal is to provide a lightweight view of commit relationships, branches, and tags without needing to leave Zed.

## Development

1. Install the extension as a dev extension in Zed (`Extensions > Install Dev Extension`) and select this directory.
2. Run `cargo build --target wasm32-wasi` to compile the WebAssembly artifact when iterating on the extension logic.
3. Restart Zed (or start it via `zed --foreground`) to see log output while debugging.

## Roadmap

- Read the current repository's commit graph and expose it as data to slash commands.
- Render a panel that displays branches and commit metadata with search/filter capabilities.
- Provide quick actions (checkout, create branch, cherry-pick) from the graph UI.

## License

MIT © Israel Oliveira
