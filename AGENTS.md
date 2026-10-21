# Agent guidelines

## Approach
- Think each change through. Build only what the current task needs, shaped so the next steps fit without rework; no speculative features, options or abstractions.
- Fogde has no users yet: keep no backward compatibility. Replace obsolete code and remove unused code and stale paths in the same change.
- Check current documentation or source before using a library, tool or API; for Herdr, the pinned version.
- Add dependencies, tools and GitHub Actions at their latest stable version; pin actions to a full commit SHA with the version in a comment.

## Working rules
- Never start the development environment or run visual checks unless asked; the user runs and tests the app.
- Run `just check` before committing or pushing and fix what it finds. Add an `#[allow]`, lint override, ignore entry or lower threshold only after the user agrees.
- Allow permissive and file-level copyleft licenses (BSD, ISC, Zlib, MPL-2.0) in cargo-deny without asking; ask before GPL, LGPL, AGPL or other strong copyleft.
- Leave changes uncommitted until the user asks. Work on `main`, without branches or pull requests unless asked.
- This file holds working rules and conventions only: product decisions go in the Fogde notes (the hub and the note it links for that area), and nothing `just check` already enforces is repeated here. When a rule changes, update it here and delete what it replaces.

## Code
- Prefer concise names, one word where it reads well. Name frontend files in kebab-case.
- Use braces for every control-flow body, put early returns on their own lines, and separate logical steps with blank lines.
- Write no comments; names explain the code. Keep only directives tools need, such as lint attributes and `// SAFETY:`.
- Never hand-edit generated files; change the source and regenerate.
- In the frontend, parse data we don't own (storage, drag data, form input) with zod schemas, and keep client state shared outside React in zustand stores.
- Keep docs short: `README.md` holds only an overview and local development instructions.

## Testing
- Test frontend and backend by risk: integrations with external tools, parsing data we don't own, connection and stream handling, security, and anything that can lose user data. Test behaviour through public interfaces with real captured data.
- Add a test that reproduces each bug before fixing it.
- Skip tests for presentation, glue and trivial code, and never require a signed-in agent.

## Rust
- Use `thiserror` in library crates and `anyhow` only in binaries. Return errors instead of panicking, and log with `tracing`.
- Use Tokio for async work and never block the runtime; move blocking work to `spawn_blocking`.
- Declare dependencies once in the root `[workspace.dependencies]`; every crate inherits `[workspace.lints]`.
- Organise the host by domain, one module per protocol area: `workspaces.rs` holds its handlers and state, `workspaces/store.rs` its SQL. Each RPC method is one `Handler` impl, registered once in its module's `register`; the router dispatches it and generates the TypeScript protocol.
- Write SQL only in a domain's `store.rs`, with sqlx's checked macros (`query!`, `query_as!`). Change the schema only with a new file in `crates/host/migrations/`, never by editing one that has shipped, and run `just sqlx` after changing a query or migration.

## Cross-platform
- Every change works on Windows, macOS and Linux; the host also runs on a VPS.
- Build paths with `Path` and `PathBuf`; never assume a separator, home folder or shell.
- Keep platform-specific code behind `#[cfg]` in one place per concern, with the same interface everywhere.

## Security
- Drive the unmodified official CLI of each agent and of GitHub (`gh`), and let users sign in through each CLI's own flow. Never read, store, log, sync or forward their credentials, or the files where those CLIs keep them.
- Keep secrets only in the OS keychain, never in files, logs or error messages.

## UI
- Use installed shadcn/ui components, installing any that are missing.
- Show loading with a spinner or skeleton, whichever fits the content.
- Show each state in one place; never repeat it across the sidebar, main area and status bar.
- Use tooltips only for icon-only controls or information not visible elsewhere; never repeat visible text.
- Keep the window chrome transparent; raise surfaces with the `surface-*` background and shadow tokens.
- App-owned spacing uses Tailwind values divisible by four, or `2`; auto margins, shadcn/ui and third-party styles are exempt.
- App-owned text uses only the `type-*` roles and the foreground colour.
- Meet WCAG 2.2 AA: every action works by keyboard with a visible focus state, and icon-only controls have accessible labels.

## Motion
- Animate only a change of state or place. Never animate keyboard-triggered actions or anything used many times a day; hover changes colour only.
- Animate only `transform` and `opacity`: ease-out to enter and leave, ease-in-out to move, never ease-in. Use the shared ease-out token and shadcn/ui's transitions as published.
- Stay under 300 ms: tooltips 125–200 ms, menus 150–250 ms, dialogs at most 300 ms, exits faster than entries. Enter from `scale-95` and `opacity-0`, never `scale-0`; popovers grow from their trigger.
- Prefer CSS transitions; use `motion/react` only for layout, presence and gestures. With reduced motion, keep fades and drop movement.
