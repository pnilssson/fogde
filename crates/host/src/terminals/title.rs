use std::path::Path;

use protocol::agents::Agent;

pub fn task(agent: Agent, title: &str) -> Option<String> {
    let stripped = strip(title);
    let task = match agent {
        Agent::Claude => {
            Some(stripped).filter(|task| *task != "Claude Code" && !Path::new(task).is_absolute())
        }
        Agent::Codex => stripped.rsplit_once(" | ").map(|(task, _)| task),
        Agent::Copilot => stripped.strip_suffix(" - GitHub Copilot"),
    }?
    .trim();

    (!task.is_empty()).then(|| task.to_owned())
}

fn strip(title: &str) -> &str {
    title.trim_start_matches(|glyph: char| {
        glyph.is_whitespace() || (!glyph.is_ascii() && !glyph.is_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    const SHELL: &str = r"C:\Program Files\PowerShell\7\pwsh.exe";
    #[cfg(unix)]
    const SHELL: &str = "/bin/zsh";

    #[test]
    fn reads_each_agents_task() {
        assert_eq!(
            [
                task(Agent::Claude, "✳ Rust crates in folder"),
                task(Agent::Codex, "Respond to greeting | app"),
                task(Agent::Copilot, "Session Initialization - GitHub Copilot"),
            ],
            [
                Some("Rust crates in folder".into()),
                Some("Respond to greeting".into()),
                Some("Session Initialization".into()),
            ]
        );
    }

    #[test]
    fn reads_claude_tasks_without_a_spinner() {
        let title = "Sidebar Claude limit display";

        assert_eq!(task(Agent::Claude, title), Some(title.into()));
    }

    #[test]
    fn ignores_titles_without_a_task() {
        assert_eq!(
            [
                task(Agent::Claude, "✳ Claude Code"),
                task(Agent::Claude, "Claude Code"),
                task(Agent::Claude, SHELL),
                task(Agent::Claude, ""),
                task(Agent::Codex, "app"),
                task(Agent::Codex, SHELL),
                task(Agent::Codex, " | app"),
                task(Agent::Copilot, "GitHub Copilot"),
            ],
            [None, None, None, None, None, None, None, None]
        );
    }

    #[test]
    fn strips_the_spinner_agents_put_before_their_title() {
        assert_eq!(
            [
                strip("✳ Rust crates in folder"),
                strip("⠂ Respond to greeting | app"),
                strip("Ärende i app"),
                strip(SHELL),
            ],
            [
                "Rust crates in folder",
                "Respond to greeting | app",
                "Ärende i app",
                SHELL
            ]
        );
    }

    #[test]
    fn keeps_separators_inside_a_codex_task() {
        assert_eq!(
            task(Agent::Codex, "Compare a | b | app"),
            Some("Compare a | b".into())
        );
    }
}
