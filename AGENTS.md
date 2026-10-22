# Agent guidelines

Before changing the frontend, read `apps/desktop/AGENTS.md`; before changing Rust, read `crates/AGENTS.md`.

## Approach
- Build only what the current task needs, shaped so the next steps fit without rework; no speculative features, options or abstractions.
- Fogde has no users yet: keep no backward compatibility, and remove obsolete code and stale paths in the same change.
- Check current documentation or source before using a library, tool or API; for Herdr, the pinned version.
- Add dependencies, tools and GitHub Actions at their latest stable version; pin actions to a full commit SHA with the version in a comment.

## Working rules
- Never start the development environment or run visual checks unless asked; the user runs and tests the app.
- Run `just check` before committing or pushing and fix what it finds. Add an `#[allow]`, lint override, ignore entry or lower threshold only after the user agrees.
- Allow permissive and file-level copyleft licenses (BSD, ISC, Zlib, MPL-2.0) in cargo-deny without asking; ask before GPL, LGPL, AGPL or other strong copyleft.
- Leave changes uncommitted until the user asks. Work on `main`, without branches or pull requests unless asked.
- Never hand-edit generated files; change the source and regenerate.
- These files hold only rules an agent would otherwise get wrong and no check enforces. Product and design decisions go in the Fogde notes. When a rule changes, replace the old one; add a new one only when the user asks.

## Code
- Prefer concise names, one word where it reads well.
- Use braces for every control-flow body, put early returns on their own lines, and separate logical steps with blank lines.
- Write no comments; names explain the code. Keep only directives tools need, such as lint attributes and `// SAFETY:`.
- Keep `README.md` to an overview and local development instructions.
- Every change works on Windows, macOS and Linux, and the host also on a Linux VPS; never assume a path separator, home folder or shell.

## Testing
- Test by risk: integrations with external tools, parsing data we don't own, connection and stream handling, security, and anything that can lose user data. Test behaviour through public interfaces with real captured data.
- Add a test that reproduces each bug before fixing it.
- Skip tests for presentation, glue and trivial code, and never require a signed-in agent.
- Use neutral sample data (`app`, `/repos/app`, `feature/login`), never the project's own name or real people's names; keep `fogde` only where it is product behaviour, such as the worktree branch prefix or the Herdr client ID.

## Security
- Drive the unmodified official CLI of each agent and of GitHub (`gh`), and let users sign in through each CLI's own flow. Never read, store, log, sync or forward their credentials, or the files where those CLIs keep them.
- Keep secrets only in the OS keychain, never in files, logs or error messages.
