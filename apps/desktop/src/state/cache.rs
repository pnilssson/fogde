#[cfg(test)]
mod tests;

use protocol::terminals::{Terminal, TerminalState};
use protocol::workspaces::{MoveTerminalParams, PlaceTerminalParams, Workspace};

use crate::workspaces::order::arranged;

#[derive(Default)]
pub struct Cache {
    list: Option<Vec<Workspace>>,
    error: Option<String>,
    latest: u64,
    loading: bool,
}

pub struct Applied {
    pub workspace: Workspace,
    pub before: Terminal,
    pub moved: bool,
}

impl Cache {
    pub fn list(&self) -> Option<&[Workspace]> {
        self.list.as_deref()
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn loading(&self) -> bool {
        self.loading
    }

    pub fn begin(&mut self) -> u64 {
        self.latest += 1;
        self.loading = true;
        self.latest
    }

    pub fn finish(&mut self, ticket: u64, result: Result<Vec<Workspace>, String>) -> bool {
        if ticket != self.latest {
            return false;
        }

        self.loading = false;
        match result {
            Ok(list) => {
                self.list = Some(list);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
        true
    }

    fn edit(&mut self, change: impl FnOnce(&mut Vec<Workspace>)) {
        if let Some(list) = &mut self.list {
            change(list);
        }
        self.latest += 1;
    }

    pub fn add(&mut self, workspace: &str, terminal: Terminal) {
        self.edit(|list| {
            if let Some(entry) = list.iter_mut().find(|entry| entry.id == workspace)
                && !entry.terminals.iter().any(|known| known.id == terminal.id)
            {
                entry.terminals.push(terminal);
            }
        });
    }

    pub fn remove(&mut self, terminal: &str) {
        self.edit(|list| {
            for entry in list {
                entry.terminals.retain(|known| known.id != terminal);
            }
        });
    }

    pub fn create(&mut self, workspace: Workspace) {
        self.edit(|list| {
            if !list.iter().any(|entry| entry.id == workspace.id) {
                list.push(workspace);
            }
        });
    }

    pub fn delete(&mut self, workspace: &str) {
        self.edit(|list| list.retain(|entry| entry.id != workspace));
    }

    pub fn apply(&mut self, state: &TerminalState) -> Option<Applied> {
        let list = self.list.as_mut()?;
        let workspace = list
            .iter_mut()
            .find(|entry| entry.terminals.iter().any(|known| known.id == state.id))?;
        let before = workspace
            .terminals
            .iter()
            .find(|known| known.id == state.id)?
            .clone();
        let snapshot = workspace.clone();

        for terminal in &mut workspace.terminals {
            if terminal.id == state.id {
                terminal.name.clone_from(&state.name);
                terminal.agent = state.agent;
                terminal.status = state.status;
                terminal.folder.clone_from(&state.folder);
                terminal.branch.clone_from(&state.branch);
                terminal.changes.clone_from(&state.changes);
            }
        }

        Some(Applied {
            moved: before.folder != state.folder,
            workspace: snapshot,
            before,
        })
    }

    pub fn place(&mut self, params: &PlaceTerminalParams) {
        let Some(list) = &self.list else {
            return;
        };
        let stays = list.iter().any(|entry| {
            entry.id == params.workspace
                && entry
                    .terminals
                    .iter()
                    .any(|known| known.id == params.terminal && known.checkout == params.checkout)
        });
        if stays {
            return;
        }

        let moved = list
            .iter()
            .flat_map(|entry| &entry.terminals)
            .find(|known| known.id == params.terminal)
            .cloned();
        self.edit(|list| {
            for entry in list {
                entry.terminals.retain(|known| known.id != params.terminal);
                if entry.id == params.workspace
                    && let Some(moved) = &moved
                {
                    entry.terminals.push(Terminal {
                        checkout: params.checkout.clone(),
                        ..moved.clone()
                    });
                }
            }
        });
    }

    pub fn arrange(&mut self, params: &MoveTerminalParams) {
        self.edit(|list| {
            for entry in list {
                let ids: Vec<String> = entry
                    .terminals
                    .iter()
                    .map(|known| known.id.clone())
                    .collect();
                if !ids.contains(&params.terminal) {
                    continue;
                }
                let order = arranged(&ids, &params.terminal, params.before.as_deref());
                entry
                    .terminals
                    .sort_by_key(|known| order.iter().position(|id| *id == known.id));
            }
        });
    }
}
