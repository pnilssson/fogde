use protocol::agents::Agent;
use protocol::terminals::Terminal;
use protocol::workspaces::{Checkout, Workspace};

use super::status::{by_attention, needs_you};
use super::tiles::tile::{Selection, View};

pub struct Group<'a> {
    pub machine: &'a str,
    pub workspaces: Option<&'a [Workspace]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waiting {
    pub view: View,
    pub terminal: Selection,
    pub name: String,
    pub agent: Option<Agent>,
    pub place: String,
}

pub fn view_of(machine: &str, workspace: &Workspace, terminal: Option<&Terminal>) -> View {
    View {
        machine: machine.to_owned(),
        id: workspace.id.clone(),
        checkout: terminal.and_then(|terminal| terminal.checkout.clone()),
    }
}

pub fn in_view(view: &View, terminal: &Terminal) -> bool {
    view.shows(terminal.checkout.as_deref())
}

pub fn worktrees(workspace: &Workspace) -> Vec<&Checkout> {
    workspace
        .checkouts
        .iter()
        .filter(|checkout| !checkout.main)
        .collect()
}

pub fn main_checkout(workspace: &Workspace) -> Option<&Checkout> {
    workspace.checkouts.iter().find(|checkout| checkout.main)
}

pub fn folder_of(workspace: &Workspace, view: &View) -> String {
    view.checkout
        .clone()
        .unwrap_or_else(|| workspace.folder.clone())
}

pub fn waiting(groups: &[Group]) -> Vec<Waiting> {
    groups
        .iter()
        .flat_map(|group| {
            group
                .workspaces
                .unwrap_or_default()
                .iter()
                .flat_map(|workspace| {
                    by_attention(&workspace.terminals)
                        .into_iter()
                        .filter(|terminal| needs_you(terminal.status))
                        .map(|terminal| Waiting {
                            view: view_of(group.machine, workspace, Some(terminal)),
                            terminal: Selection::new(group.machine, terminal.id.clone()),
                            name: terminal.name.clone(),
                            agent: terminal.agent,
                            place: workspace.name.clone(),
                        })
                })
        })
        .collect()
}

pub fn next_waiting(groups: &[Group], active: Option<&Selection>) -> Option<Waiting> {
    let list = waiting(groups);
    if list.is_empty() {
        return None;
    }
    let at = list
        .iter()
        .position(|waiting| Some(&waiting.terminal) == active);
    let next = at.map_or(0, |at| (at + 1) % list.len());
    list.into_iter().nth(next)
}

pub fn resolve(groups: &[Group], stored: Option<&View>) -> Option<View> {
    if let Some(stored) = stored
        && let Some(group) = groups.iter().find(|group| group.machine == stored.machine)
    {
        let Some(workspaces) = group.workspaces else {
            return Some(stored.clone());
        };
        if let Some(workspace) = workspaces
            .iter()
            .find(|workspace| workspace.id == stored.id)
        {
            let known = worktrees(workspace)
                .iter()
                .any(|checkout| Some(&checkout.path) == stored.checkout.as_ref());
            if stored.checkout.is_some() && !known {
                return Some(stored.home());
            }
            return Some(stored.clone());
        }
    }

    groups.iter().find_map(|group| {
        group
            .workspaces
            .and_then(|workspaces| workspaces.first())
            .map(|first| View::workspace(group.machine, first.id.clone()))
    })
}

#[cfg(test)]
mod tests {
    use protocol::terminals::Status;

    use super::*;

    fn terminal(id: &str, status: Status) -> Terminal {
        Terminal {
            id: id.into(),
            name: id.into(),
            agent: Some(Agent::Claude),
            status,
            folder: String::new(),
            branch: None,
            changes: None,
            checkout: None,
        }
    }

    fn workspace(id: &str, terminals: Vec<Terminal>) -> Workspace {
        Workspace {
            id: id.into(),
            name: id.into(),
            folder: String::new(),
            missing: false,
            checkouts: Vec::new(),
            terminals,
        }
    }

    fn fixture() -> (Vec<Workspace>, Vec<Workspace>) {
        (
            vec![workspace(
                "api",
                vec![
                    terminal("a", Status::Idle),
                    terminal("b", Status::Done),
                    terminal("c", Status::Blocked),
                ],
            )],
            vec![workspace("web", vec![terminal("d", Status::Done)])],
        )
    }

    fn next(groups: &[Group], active: Option<(&str, &str)>) -> Option<String> {
        let active = active.map(|(machine, id)| Selection::new(machine, id));
        next_waiting(groups, active.as_ref()).map(|waiting| waiting.terminal.id)
    }

    #[test]
    fn goes_to_each_agent_that_needs_you_in_sidebar_order_and_wraps_around() {
        let (local, vps) = fixture();
        let groups = [
            Group {
                machine: "local",
                workspaces: Some(&local),
            },
            Group {
                machine: "vps",
                workspaces: Some(&vps),
            },
        ];

        assert_eq!(next(&groups, None).as_deref(), Some("b"));
        assert_eq!(next(&groups, Some(("local", "a"))).as_deref(), Some("b"));
        assert_eq!(next(&groups, Some(("local", "b"))).as_deref(), Some("c"));
        assert_eq!(next(&groups, Some(("local", "c"))).as_deref(), Some("d"));
        assert_eq!(next(&groups, Some(("vps", "d"))).as_deref(), Some("b"));
    }

    #[test]
    fn a_removed_worktrees_view_falls_back_to_its_workspace() {
        let (local, _) = fixture();
        let groups = [Group {
            machine: "local",
            workspaces: Some(&local),
        }];
        let stored = View {
            machine: "local".into(),
            id: "api".into(),
            checkout: Some("/worktrees/login".into()),
        };

        assert_eq!(
            resolve(&groups, Some(&stored)),
            Some(View::workspace("local", "api"))
        );
    }

    #[test]
    fn finds_nothing_when_no_agent_needs_you() {
        let idle = vec![workspace("api", vec![terminal("a", Status::Working)])];
        let groups = [Group {
            machine: "local",
            workspaces: Some(&idle),
        }];

        assert_eq!(next(&groups, None), None);
    }
}
