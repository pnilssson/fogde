# Rust

- Use `thiserror` in library crates and `anyhow` only in binaries. Return errors instead of panicking, and log with `tracing`.
- Use Tokio for async work and never block the runtime; move blocking work to `spawn_blocking`.
- Declare dependencies once in the root `[workspace.dependencies]`; every crate inherits `[workspace.lints]`.
- Organise the host by domain, one module per protocol area: `workspaces.rs` holds its handlers and state, `workspaces/store.rs` its SQL. Each RPC method is one `Handler` impl, registered once in its module's `register`; the router dispatches it.
- Write SQL only in a domain's `store.rs`, with sqlx's checked macros (`query!`, `query_as!`). Change the schema only with a new file in `crates/host/migrations/`, never by editing one that has shipped, and run `just sqlx` after changing a query or migration.
- Build paths with `Path` and `PathBuf`, and keep platform-specific code behind `#[cfg]` in one place per concern, with the same interface everywhere.
