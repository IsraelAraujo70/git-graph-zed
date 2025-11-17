# Git Graph Extension Plan

## Objective
Deliver a native Git history visualization for Zed that mirrors the usefulness of `git log --graph` while offering richer interactivity (branch filters, quick actions).

## Milestones

1. **Repository Data Pipeline**
   - Determine the active workspace root via `zed::workspace`.
   - Execute `git log --graph --pretty=...` via `zed::Command` (or libgit2 wrapper) and parse the ASCII graph.
   - Normalize commits into nodes/edges, calculating branch/tag metadata.

2. **Graph Rendering MVP**
   - Provide a custom view (panel or sheet) powered by HTML Canvas/WebView.
   - Render commits as colored nodes with branch lines mirroring CLI output.
   - Display commit metadata tooltip on hover and detail pane on selection.

3. **Interaction Layer**
   - Implement filters (branch, author, time).
   - Add quick actions (checkout, create branch, copy SHA) exposed via slash commands/context menu.
   - Keep view in sync by refreshing when repository changes (watch `.git/HEAD`).

4. **Performance & Polish**
   - Debounce refreshes, virtualize large histories (>3k commits).
   - Persist UI state per workspace.
   - Localization, accessibility (keyboard navigation, screen reader-friendly summaries).

5. **Release**
   - Document usage in `README.md` with screenshots/GIF.
   - Provide configuration knobs in `extension.toml`.
   - Prepare changelog and publish to `zed-industries/extensions`.

## Risks & Mitigations
- Large repos: limit history depth or incremental load.
- Platform git availability: fall back to Zed API provided commit info once exposed.
- UI complexity: start with simple column-based ASCII mimic before moving to full graph.
