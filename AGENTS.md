# Agent guidelines

## Approach
- Think each change through before writing it. Build only what the current task needs, but shape it so the next steps fit without rework. Add no speculative features, options or abstractions.
- Fogde has no users yet: do not preserve backward compatibility. Replace obsolete code, and remove unused code and stale paths in the same change.
- Check current documentation or source before using a library, tool or API instead of relying on memory. For Herdr, use the pinned version.
- Add dependencies, tools and GitHub Actions at their latest stable version. Pin actions to a full commit SHA with the version in a comment.

## Working rules
- Never start the development environment or run browser-based visual checks unless asked; the user runs and tests the app.
- Run `just check` before committing or pushing, not after every change.
- Fix what the checks find instead of silencing it. Add an `#[allow]`, an ignore entry, a lower threshold or an allowed license only after the user agrees.
- Leave changes uncommitted until the user asks. Work on `main`, without branches or pull requests unless asked.
- When the user sets or changes a rule, update this file in the same change and delete what it replaces.

## Code style
- Prefer concise names without unnecessary hyphens.
- Always use braces for control-flow bodies, put early returns on their own lines, and separate logical steps with blank lines.
- Write no comments in our own code; names explain it. Keep only directives tools need, such as lint attributes and `// SAFETY:`.
- Keep files under 300 lines by splitting along responsibilities.
- Never hand-edit generated files; change the source and regenerate.
- Keep docs short. `README.md` holds only an overview and local development instructions.

## Rust
- Use `thiserror` in library crates and `anyhow` only in binaries. Return errors instead of panicking, and log with `tracing`.
- Use Tokio for async work and never block the runtime; move blocking work to `spawn_blocking`.
- Declare dependencies once in the root `[workspace.dependencies]`; every crate inherits `[workspace.lints]`.
- Write `unsafe` code only after the user agrees.

## Cross-platform
- Every change works on Windows, macOS and Linux (the host can run on a VPS).
- Build paths with `Path` and `PathBuf`; never assume a separator, home directory or shell.
- Keep platform-specific code behind `#[cfg]` in one place per concern, with the same interface everywhere.

## Security
- Drive the unmodified `claude` and `codex` binaries and let users sign in through each CLI's own flow. Never read, store, log, sync or forward their credentials.
- Keep secrets only in the OS keychain, never in files, logs or error messages.

## UI
- Use installed shadcn/ui components for controls, installing any that are missing. Never edit them or override their appearance unless the user agrees.
- Show loading with a spinner or skeleton, whichever fits the content.
- App-owned padding, margins and gaps use Tailwind spacing values divisible by four, or `2`. Auto margins, shadcn/ui and third-party styles are exempt.
- App-owned text uses only the shared typography roles, never its own sizes, weights, families, line heights or letter spacing. Discuss a new role before adding one.
- Use Phosphor (`@phosphor-icons/react`) for every icon, and show state with its weights (`regular` idle, `fill` active) rather than extra icons or badges.
- Meet WCAG 2.2 AA: every action works by keyboard with a visible focus state, and icon-only controls have accessible labels.

## Motion
- Animate only to show a change of state or place. Never animate keyboard-triggered actions or anything used many times a day; hover changes colour only.
- Animate only `transform` and `opacity`: ease-out for entering and leaving, ease-in-out for movement, never ease-in. Use the shared ease-out token and shadcn/ui's built-in transitions as published.
- Stay under 300 ms: tooltips 125–200 ms, menus 150–250 ms, dialogs at most 300 ms, exits faster than entries. Enter from `scale-95` and `opacity-0`, never `scale-0`; popovers grow from their trigger.
- Prefer CSS transitions; use `motion/react` only for layout, presence and gestures. With reduced motion, keep fades and drop movement.
