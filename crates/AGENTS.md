# Rust

- Use `thiserror` in library crates and `anyhow` only in binaries. Return errors instead of panicking, and log with `tracing`.
- Use Tokio for async work and never block the runtime; move blocking work to `spawn_blocking`.
- Declare dependencies once in the root `[workspace.dependencies]`; every crate inherits `[workspace.lints]`.
- Organise the host by domain, one module per protocol area: `workspaces.rs` holds its handlers and state, `workspaces/store.rs` its SQL. Each RPC method is one `Handler` impl, registered once in its module's `register`; the router dispatches it and generates the TypeScript protocol.
- Write SQL only in a domain's `store.rs`, with sqlx's checked macros (`query!`, `query_as!`). Databases owned by other tools, such as an agent's session store, are the exception: open them read-only and query them with unchecked `sqlx::query_as` in the module that reads them. Change the schema only with a new file in `crates/host/migrations/`, never by editing one that has shipped.
- Build paths with `Path` and `PathBuf`, and keep platform-specific code behind `#[cfg]` in one place per concern, with the same interface everywhere.
- Test Linux-only code (the server host, `#[cfg(unix)]`) in Docker instead of waiting for CI: `cargo test` in `rust:1-bookworm` with the repo mounted read-only and a named volume as `CARGO_TARGET_DIR`. To prove a test reproduces a bug, run it against the old code in a copy (sources only) with its own target volume, since `fogde-host` lands at the same unhashed path. Remove the volumes and image afterwards.
