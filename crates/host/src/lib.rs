mod accessibility;
mod agents;
#[cfg(unix)]
mod daemon;
mod db;
mod folders;
mod github;
mod health;
mod logs;
mod machines;
mod notifications;
mod onboarding;
mod relay;
mod repos;
mod rpc;
mod server;
mod session;
mod settings;
mod shared;
mod socket;
mod stats;
mod terminals;
mod theme;
mod token;
mod usage;
mod workspaces;
mod worktrees;

#[cfg(unix)]
pub use daemon::{proxy, serve};
pub use logs::log;
pub use remote::Binaries;
pub use shared::fogde;
pub use socket::start;
pub use usage::report;
