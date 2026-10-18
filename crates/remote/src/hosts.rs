use tokio::fs;

pub async fn hosts() -> Vec<String> {
    let Some(config) = std::env::home_dir().map(|home| home.join(".ssh").join("config")) else {
        return Vec::new();
    };

    fs::read_to_string(config)
        .await
        .map(|text| parse(&text))
        .unwrap_or_default()
}

fn parse(config: &str) -> Vec<String> {
    let mut hosts: Vec<String> = Vec::new();

    for line in config.lines() {
        let mut words = line
            .split(|c: char| c.is_whitespace() || c == '=')
            .filter(|word| !word.is_empty());
        if !words
            .next()
            .is_some_and(|key| key.eq_ignore_ascii_case("host"))
        {
            continue;
        }

        for name in words {
            if !name.contains(['*', '?', '!']) && !hosts.iter().any(|host| host == name) {
                hosts.push(name.to_owned());
            }
        }
    }

    hosts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_named_hosts_and_skips_patterns() {
        let config = "Host *\n  ServerAliveInterval 30\n\nHost my-vps work\n  HostName 1.2.3.4\nhost=staging\nHost !bastion github.com\n  User git\nHost my-vps\nMatch host foo\n";

        assert_eq!(parse(config), ["my-vps", "work", "staging", "github.com"]);
    }
}
