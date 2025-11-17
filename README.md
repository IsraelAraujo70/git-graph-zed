# Git Graph (Zed Extension)

Git Graph is a work-in-progress Zed extension that aims to bring an interactive Git history visualization to the editor. The goal is to provide a lightweight view of commit relationships, branches, and tags without needing to leave Zed.

## Development

1. Install the extension as a dev extension in Zed (`Extensions > Install Dev Extension`) and select this directory.
2. Run `cargo build --target wasm32-wasi` to compile the WebAssembly artifact when iterating on the extension logic.
3. Restart Zed (or start it via `zed --foreground`) to see log output while debugging.

## Slash Command

- Use `/git-graph [limit]` inside the Assistant to capture the current worktree history. The optional `limit` (default 400, max 2000) controls how many commits are streamed from `git log`.
- The command returns a JSON payload containing commit nodes, edges, branch/tag metadata, and whether the graph was truncated. This acts as the data contract for the upcoming visual panel.

## WebView Prototype

- The `webview/` folder hosts a lightweight HTML/CSS/JS prototype that visualizes the Slash Command output.
- During development run `python3 -m http.server --directory webview 4173` (or any static server) and open `http://localhost:4173`.
- Drop a JSON file produced by `/git-graph` onto the page or rely on the bundled `sample-data.json`. The UI also posts a `git-graph:request` message so a future in-editor WebView can provide live data.

### Current Status / On Hold

Zed’s extension API (0.1.x) only exposes language servers and slash-command surfaces. There is no public way for an extension to register a persistent panel, spawn a WebView, or auto-open UI when the extension loads. Because of this limitation the Git Graph visualization cannot ship as an interactive panel yet—the slash command is the only viable in-editor surface. The project will stay on hold until Zed releases APIs for custom panels or webviews; once available the `webview/` prototype can be embedded directly and fed by the existing data pipeline.

## Roadmap

- Read the current repository's commit graph and expose it as data to slash commands.
- Render a panel that displays branches and commit metadata with search/filter capabilities.
- Provide quick actions (checkout, create branch, cherry-pick) from the graph UI.

## License

MIT © Israel Oliveira
