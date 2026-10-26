use protocol::terminals::{Status, Terminal};

struct Traits {
    needs_you: bool,
    busy: bool,
    rank: u8,
    activity: &'static str,
    label: Option<&'static str>,
}

fn traits(status: Status) -> Traits {
    match status {
        Status::Idle => Traits {
            needs_you: false,
            busy: false,
            rank: 0,
            activity: "is open",
            label: None,
        },
        Status::Working => Traits {
            needs_you: false,
            busy: true,
            rank: 1,
            activity: "is still working",
            label: Some("Working"),
        },
        Status::Done => Traits {
            needs_you: true,
            busy: false,
            rank: 2,
            activity: "has finished",
            label: Some("Finished"),
        },
        Status::Blocked => Traits {
            needs_you: true,
            busy: true,
            rank: 3,
            activity: "is waiting for your input",
            label: Some("Needs your input"),
        },
    }
}

pub fn needs_you(status: Status) -> bool {
    traits(status).needs_you
}

pub fn busy(status: Status) -> bool {
    traits(status).busy
}

pub fn activity(status: Status) -> &'static str {
    traits(status).activity
}

pub fn label(status: Status) -> Option<&'static str> {
    traits(status).label
}

pub fn attention(terminals: &[Terminal]) -> Status {
    terminals
        .iter()
        .map(|terminal| terminal.status)
        .filter(|status| needs_you(*status))
        .fold(Status::Idle, |top, status| {
            if traits(status).rank > traits(top).rank {
                status
            } else {
                top
            }
        })
}

pub fn by_attention(terminals: &[Terminal]) -> Vec<&Terminal> {
    let mut sorted: Vec<&Terminal> = terminals.iter().collect();
    sorted.sort_by_key(|terminal| !needs_you(terminal.status));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal(status: Status) -> Terminal {
        Terminal {
            id: "w1".into(),
            name: "Claude".into(),
            agent: None,
            status,
            folder: "/repos/app".into(),
            branch: None,
            changes: None,
            checkout: None,
        }
    }

    #[test]
    fn a_workspace_shows_its_most_urgent_terminal() {
        let terminals = [
            terminal(Status::Working),
            terminal(Status::Done),
            terminal(Status::Blocked),
        ];
        assert_eq!(attention(&terminals), Status::Blocked);
        assert_eq!(attention(&[terminal(Status::Working)]), Status::Idle);
    }

    #[test]
    fn blocked_and_done_need_you_while_working_and_blocked_are_busy() {
        assert!(needs_you(Status::Done) && needs_you(Status::Blocked));
        assert!(!needs_you(Status::Working));
        assert!(busy(Status::Working) && busy(Status::Blocked));
        assert!(!busy(Status::Done));
    }
}
