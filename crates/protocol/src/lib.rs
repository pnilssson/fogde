pub mod agents;
mod connection;
pub mod folders;
mod frame;
pub mod github;
pub mod machines;
mod method;
pub mod onboarding;
pub mod repos;
pub mod rpc;
pub mod server;
pub mod terminals;
pub mod theme;
pub mod workspaces;
pub mod worktrees;

pub use connection::Connection;
pub use frame::{Frame, Platform};
pub use method::{Event, Method};
