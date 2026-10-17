use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeStatus {
    logged_in: bool,
}

pub fn claude(stdout: &[u8]) -> Option<bool> {
    serde_json::from_slice::<ClaudeStatus>(stdout)
        .ok()
        .map(|status| status.logged_in)
}

pub fn codex(success: bool, stderr: &[u8]) -> Option<bool> {
    if success {
        return Some(true);
    }

    String::from_utf8_lossy(stderr)
        .lines()
        .any(|line| line.trim() == "Not logged in")
        .then_some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_claude_status() {
        let signed_in = br#"{
  "loggedIn": true,
  "authMethod": "claude.ai",
  "apiProvider": "firstParty",
  "analyticsDisabled": false,
  "projectsDirectory": "C:\\Users\\dev\\.claude\\projects",
  "configDirectory": "C:\\Users\\dev\\.claude",
  "email": "dev@example.com",
  "orgId": "00000000-0000-0000-0000-000000000000",
  "orgName": "dev@example.com's Organization",
  "subscriptionType": "pro"
}"#;
        let signed_out = br#"{
  "loggedIn": false,
  "authMethod": "none",
  "apiProvider": "firstParty",
  "analyticsDisabled": false,
  "projectsDirectory": "C:\\Users\\dev\\.claude\\projects",
  "configDirectory": "C:/Users/dev/.claude"
}"#;

        assert_eq!(claude(signed_in), Some(true));
        assert_eq!(claude(signed_out), Some(false));
        assert_eq!(claude(b"error: unknown command 'auth'"), None);
    }

    #[test]
    fn reads_codex_status() {
        let signed_out =
            b"WARNING: proceeding, even though we could not create PATH aliases\nNot logged in\n";

        assert_eq!(codex(true, b"Logged in using ChatGPT\n"), Some(true));
        assert_eq!(codex(false, signed_out), Some(false));
        assert_eq!(codex(false, b"Error loading configuration"), None);
    }
}
