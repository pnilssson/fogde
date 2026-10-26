use serde::{Deserialize, Serialize};

use crate::Method;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateWorktreeParams {
    pub repo: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoveWorktreeParams {
    pub repo: String,
    pub path: String,
}

pub struct CreateWorktree;

impl Method for CreateWorktree {
    const NAME: &'static str = "worktrees.create";
    type Params = CreateWorktreeParams;
    type Result = String;
}

pub struct RemoveWorktree;

impl Method for RemoveWorktree {
    const NAME: &'static str = "worktrees.remove";
    type Params = RemoveWorktreeParams;
    type Result = ();
}
