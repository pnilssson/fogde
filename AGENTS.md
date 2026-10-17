# Agent guidelines

## Working rules
- Never start or run the development environment. The user runs it and tests locally.
- The user verifies UI changes; do not run browser-based visual checks unless asked.
- Do not run checks after every change. Run `just check` before committing or pushing.
- Fix what the checks find instead of silencing it. Add an `#[allow]`, an ignore entry, a lower threshold or a license to the allowlist only after the user agrees.
- Leave changes uncommitted until the user asks for a commit. The user reviews them locally and iterates first.
- Fogde has no users yet, so work directly on `main`. Do not create branches or pull requests unless the user asks.
- Tauri 2, Herdr and the agent CLIs change quickly. Check their current documentation before writing integration code instead of relying on memory, and read the docs for the pinned Herdr version.
- Add every dependency, tool and GitHub Action at its latest stable version, checked in its registry rather than from memory. Pin GitHub Actions to a full commit SHA with the version in a comment.
- When the user sets or changes a rule, update this file in the same change. Delete stale rules instead of adding contradictory ones.

## Code style
- Prefer concise names without unnecessary hyphens for all identifiers.
- Always use braces for control-flow bodies. Put early returns on their own lines, and separate logical steps with blank lines.
- Do not write comments in our own code; name things so the code explains itself. Keep only directives that tools need, such as lint attributes and `// SAFETY:` comments on `unsafe` blocks.
- Keep files under 300 lines; `just check` enforces it. Split along responsibilities, not at arbitrary points.
- Never hand-edit generated files, such as TypeScript types generated from Rust or Herdr's API types. Change the source and regenerate.
- Fogde has no users yet, so do not preserve backward compatibility. Replace obsolete code when that gives a cleaner foundation, and remove unused code and stale paths as part of the change.
- Prefer self-explanatory code to separate documentation. Keep `README.md` brief, limited to a project overview and local development instructions.

## Rust
- Use `thiserror` for errors in library crates and `anyhow` only in binaries.
- Never use `unwrap`, `expect`, `panic!`, `todo!` or `unimplemented!` outside tests; return errors instead.
- Log with `tracing`. Never use `println!`, `eprintln!` or `dbg!`.
- Use Tokio for async work and never block the runtime; move blocking work to `spawn_blocking`.
- Declare dependencies once in the root `[workspace.dependencies]`, and let every crate inherit the root `[workspace.lints]`.
- Write `unsafe` code only after the user agrees.

## Cross-platform
- Fogde runs on Windows, macOS and Linux (the host can run on a VPS). Every change must work on all three.
- Build paths with `Path` and `PathBuf`. Never concatenate path strings or assume a separator, home directory or shell.
- Keep platform-specific code behind `#[cfg]` in one place per concern, with the same interface on every platform.

## Security
- Drive the unmodified `claude` and `codex` binaries, and let users sign in through each CLI's own flow.
- Never read, store, log, sync or forward Claude Code or Codex credentials, and never extract their OAuth tokens.
- Keep secrets only in the OS keychain. Never write tokens to files, logs or error messages.

## UI and styling
- Use installed shadcn/ui components for UI controls. If a suitable component is not installed, install it.
- Never edit installed shadcn/ui components unless the user explicitly asks you to.
- Do not override the appearance of shadcn/ui components with custom class names unless the user has agreed to the specific styling change.
- Use spinners or skeletons for loading states, choosing whichever best fits the content and interaction.
- Use Tailwind spacing values divisible by four, or `2`, for app-owned padding, margins, and gaps, including variants, negative values, and `space-*`/`gap-*`. Allow auto margins and `space-*-reverse`. Exempt shadcn/ui, third-party styles, sizes, and positioning.
- Use only the shared typography roles for app-owned text. Never add text sizes, weights, families, line heights or letter spacing elsewhere. Preserve shadcn/ui typography, and discuss a new role before adding one.
- Use Phosphor (`@phosphor-icons/react`) for every interface icon. Show state with its weights, such as `regular` for idle and `fill` for active, rather than with extra icons or badges.

## Motion
- Use shadcn/ui's built-in transitions as published.
- Add motion only to show a change of state or place, never as decoration. Never animate keyboard-triggered actions or anything used many times a day; hover changes colour only.
- Animate only `transform` and `opacity`. Use ease-out for entering and leaving and ease-in-out for movement on screen, never ease-in. Use the shared strong ease-out token instead of default curves.
- Keep interface motion under 300 ms: tooltips 125–200 ms, menus 150–250 ms, dialogs at most 300 ms, and exits faster than entries. Enter from `scale-95` and `opacity-0`, never from `scale-0`; popovers grow from their trigger.
- Prefer CSS transitions; use `motion/react` only for layout, presence and gesture animation.
- With reduced motion, keep fades and drop movement.

## Accessibility
- Meet applicable WCAG 2.2 AA criteria from the start of each UI change.
- Keep every action reachable by keyboard with a visible focus state, and give icon-only controls accessible labels.
