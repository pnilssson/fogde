mod branches;
mod changes;
mod commit;
mod diff;
mod error;
mod head;
mod history;
mod include;
mod remote;
mod run;
mod status;
#[cfg(test)]
mod testing;
mod worktree;
mod write;

pub use branches::{Branch, branches};
pub use changes::{Base, Changes, Divergence, changes, fetch};
pub use commit::commit;
pub use diff::{Diff, FileDiff, Scope, diff};
pub use error::Error;
pub use head::{branch, common, heads, root};
pub use history::{LastCommit, last, undo};
pub use remote::{Config, Remote, config, remotes};
pub use status::{Change, File, Status, status};
pub use worktree::{
    Worktree, add_detached, add_worktree, clean, discard_worktree, remove_merged_branch,
    remove_worktree, worktrees,
};
pub use write::{fetched, pull, push, refresh, switch};
