use gpui_kit::{Context, Task};
use protocol::Method;
use protocol::terminals::{CloseTerminal, CreateTerminal, CreateTerminalParams, Terminal};
use protocol::workspaces::{CreateWorkspace, DeleteWorkspace, Workspace};
use protocol::worktrees::{
    CreateWorktree, CreateWorktreeParams, RemoveWorktree, RemoveWorktreeParams,
};

use super::Workspaces;
use crate::notice;
use crate::paths;
use crate::runtime;
use crate::workspaces::selection::main_checkout;

impl Workspaces {
    fn request<M>(
        &mut self,
        machine: &str,
        params: M::Params,
        failure: String,
        cx: &mut Context<Self>,
    ) -> Task<Option<M::Result>>
    where
        M: Method,
        M::Params: Send + Sync,
        M::Result: Send,
    {
        let Some(client) = self
            .machines
            .read(cx)
            .get(machine)
            .map(|machine| machine.client.clone())
        else {
            return Task::ready(None);
        };
        let request = runtime::call::<M>(&client, params, cx);
        cx.spawn(async move |_, cx| match request.await {
            Ok(result) => Some(result),
            Err(error) => {
                cx.update(|cx| notice::error(failure, &error, cx));
                None
            }
        })
    }

    pub fn create_terminal(
        &mut self,
        machine: &str,
        params: CreateTerminalParams,
        cx: &mut Context<Self>,
    ) -> Task<Option<Terminal>> {
        let workspace = params.workspace.clone();
        let request =
            self.request::<CreateTerminal>(machine, params, "Could not open a terminal".into(), cx);
        let machine = machine.to_owned();
        cx.spawn(async move |this, cx| {
            let terminal = request.await?;
            this.update(cx, |this, cx| {
                this.add(&machine, &workspace, terminal.clone(), cx);
            })
            .ok();
            Some(terminal)
        })
    }

    pub fn closing(&self, machine: &str, terminal: &str) -> bool {
        self.closing
            .iter()
            .any(|(known, id)| known == machine && id == terminal)
    }

    pub fn close_terminal(&mut self, machine: &str, terminal: &Terminal, cx: &mut Context<Self>) {
        let key = (machine.into(), terminal.id.clone());
        if self.closing.contains(&key) {
            return;
        }
        self.closing.insert(key.clone());
        cx.notify();

        let failure = format!("Could not close {}", terminal.name);
        let request = self.request::<CloseTerminal>(machine, terminal.id.clone(), failure, cx);
        cx.spawn(async move |this, cx| {
            let closed = request.await.is_some();
            this.update(cx, |this, cx| {
                this.closing.remove(&key);
                if closed {
                    this.remove(&key.0, &key.1, cx);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn create_workspace(
        &mut self,
        machine: &str,
        folder: String,
        cx: &mut Context<Self>,
    ) -> Task<Option<Workspace>> {
        let request = self.request::<CreateWorkspace>(
            machine,
            folder,
            "Could not create the workspace".into(),
            cx,
        );
        let machine = machine.to_owned();
        cx.spawn(async move |this, cx| {
            let workspace = request.await?;
            let created = workspace.clone();
            this.update(cx, |this, cx| {
                this.edit(&machine, |cache| cache.create(created), cx);
            })
            .ok();
            Some(workspace)
        })
    }

    pub fn delete_workspace(
        &mut self,
        machine: &str,
        workspace: &str,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let request = self.request::<DeleteWorkspace>(
            machine,
            workspace.to_owned(),
            "Could not delete the workspace".into(),
            cx,
        );
        let (machine, workspace) = (machine.to_owned(), workspace.to_owned());
        cx.spawn(async move |this, cx| {
            let deleted = request.await.is_some();
            this.update(cx, |this, cx| {
                if deleted {
                    this.edit(&machine, |cache| cache.delete(&workspace), cx);
                } else {
                    this.reload(&machine, cx);
                }
            })
            .ok();
            deleted
        })
    }

    pub fn create_worktree(
        &mut self,
        machine: &str,
        params: CreateWorktreeParams,
        cx: &mut Context<Self>,
    ) -> Task<Option<String>> {
        let request = self.request::<CreateWorktree>(
            machine,
            params,
            "Could not create the worktree".into(),
            cx,
        );
        let machine = machine.to_owned();
        cx.spawn(async move |this, cx| {
            let path = request.await?;
            this.update(cx, |this, cx| this.reload(&machine, cx)).ok();
            Some(path)
        })
    }

    pub fn remove_worktree(
        &mut self,
        machine: &str,
        workspace: &Workspace,
        path: &str,
        cx: &mut Context<Self>,
    ) -> Task<bool> {
        let refusal = match main_checkout(workspace) {
            None => Some("The repository's main checkout is missing."),
            Some(_) if busy(workspace, path) => Some("Close its terminals to remove it."),
            Some(_) => None,
        };
        if let Some(refusal) = refusal {
            notice::error("Could not remove the worktree", &refusal, cx);
            return Task::ready(false);
        }
        let key = (machine.into(), path.to_owned());
        if !self.removing.insert(key.clone()) {
            return Task::ready(false);
        }

        let params = RemoveWorktreeParams {
            repo: main_checkout(workspace)
                .map(|main| main.path.clone())
                .unwrap_or_default(),
            path: path.to_owned(),
        };
        let request = self.request::<RemoveWorktree>(
            machine,
            params,
            "Could not remove the worktree".into(),
            cx,
        );
        cx.spawn(async move |this, cx| {
            let removed = request.await.is_some();
            this.update(cx, |this, cx| {
                this.removing.remove(&key);
                this.reload(&key.0, cx);
            })
            .ok();
            removed
        })
    }
}

pub fn busy(workspace: &Workspace, checkout: &str) -> bool {
    workspace.terminals.iter().any(|terminal| {
        terminal.checkout.as_deref() == Some(checkout) || paths::inside(&terminal.folder, checkout)
    })
}
