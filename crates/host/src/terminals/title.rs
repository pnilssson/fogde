use protocol::agents::Agent;

pub fn strip(title: &str) -> &str {
    title.trim_start_matches(|glyph: char| {
        glyph.is_whitespace() || (!glyph.is_ascii() && !glyph.is_alphanumeric())
    })
}

pub fn task(agent: Agent, title: &str, stripped: &str) -> Option<String> {
    let task = match agent {
        Agent::Claude => Some(stripped).filter(|task| *task != title && *task != "Claude Code"),
        Agent::Codex => stripped.rsplit_once(" | ").map(|(task, _)| task),
        Agent::Copilot => stripped.strip_suffix(" - GitHub Copilot"),
    }?
    .trim();

    (!task.is_empty()).then(|| task.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_agents_task() {
        assert_eq!(
            [
                task(
                    Agent::Claude,
                    "✳ Rust crates in folder",
                    "Rust crates in folder"
                ),
                task(
                    Agent::Codex,
                    "Respond to greeting | app",
                    "Respond to greeting | app"
                ),
                task(
                    Agent::Copilot,
                    "Session Initialization - GitHub Copilot",
                    "Session Initialization - GitHub Copilot"
                ),
            ],
            [
                Some("Rust crates in folder".into()),
                Some("Respond to greeting".into()),
                Some("Session Initialization".into()),
            ]
        );
    }

    #[test]
    fn ignores_titles_without_a_task() {
        let shell = r"C:\Program Files\PowerShell\7\pwsh.exe";

        assert_eq!(
            [
                task(Agent::Claude, "✳ Claude Code", "Claude Code"),
                task(Agent::Claude, shell, shell),
                task(Agent::Codex, "app", "app"),
                task(Agent::Codex, shell, shell),
                task(Agent::Codex, " | app", " | app"),
                task(Agent::Copilot, "GitHub Copilot", "GitHub Copilot"),
            ],
            [None, None, None, None, None, None]
        );
    }

    #[test]
    fn strips_the_spinner_agents_put_before_their_title() {
        let shell = r"C:\Program Files\PowerShell\7\pwsh.exe";

        assert_eq!(
            [
                strip("✳ Rust crates in folder"),
                strip("⠂ Respond to greeting | app"),
                strip("Ärende i app"),
                strip(shell),
            ],
            [
                "Rust crates in folder",
                "Respond to greeting | app",
                "Ärende i app",
                shell
            ]
        );
    }

    #[test]
    fn keeps_separators_inside_a_codex_task() {
        assert_eq!(
            task(Agent::Codex, "Compare a | b | app", "Compare a | b | app"),
            Some("Compare a | b".into())
        );
    }
}
