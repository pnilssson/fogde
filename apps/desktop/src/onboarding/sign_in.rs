fn host_of(url: &str) -> Option<String> {
    let rest = url
        .get(..8)
        .filter(|scheme| scheme.eq_ignore_ascii_case("https://"))
        .map(|_| &url[8..])?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    let host = match host.rsplit_once(':') {
        Some((name, port)) if port.chars().all(|char| char.is_ascii_digit()) => name,
        _ => host,
    };
    let valid = !host.is_empty()
        && host
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '.');
    valid.then(|| host.to_ascii_lowercase())
}

pub fn trusted(url: &str, domains: &[&str]) -> bool {
    let Some(host) = host_of(url.trim()) else {
        return false;
    };
    domains.iter().any(|domain| {
        let domain = domain.to_ascii_lowercase();
        host == domain || host.ends_with(&format!(".{domain}"))
    })
}

pub fn hostname(input: &str) -> String {
    let host = input.trim();
    let lower = host.to_ascii_lowercase();
    let url = if lower.starts_with("https://") {
        host.to_owned()
    } else if lower.starts_with("http://") {
        format!("https://{}", &host[7..])
    } else {
        format!("https://{host}")
    };
    host_of(&url).unwrap_or_else(|| host.to_owned())
}

pub fn untrusted(url: &str) -> String {
    format!("The sign-in link went to an unexpected address: {url}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAUDE: &[&str] = &["claude.com", "claude.ai"];

    #[test]
    fn trusts_https_links_on_a_known_domain_or_its_subdomains() {
        assert!(trusted(
            "https://claude.com/cai/oauth/authorize?state=abc",
            CLAUDE
        ));
        assert!(trusted(
            "https://auth.openai.com/codex/device",
            &["openai.com"]
        ));
    }

    #[test]
    fn rejects_other_hosts_imitations_and_other_schemes() {
        assert!(!trusted("https://evil.example/claude.com", CLAUDE));
        assert!(!trusted("https://claude.com.evil.example/", CLAUDE));
        assert!(!trusted("https://notclaude.com/", CLAUDE));
        assert!(!trusted("https://claude.com@evil.example/", CLAUDE));
        assert!(!trusted("http://claude.com/", CLAUDE));
        assert!(!trusted("file:///C:/Windows/System32/calc.exe", CLAUDE));
        assert!(!trusted("not a url", CLAUDE));
    }

    #[test]
    fn trusts_only_the_github_enterprise_host_the_user_signs_in_to() {
        for input in ["acme.ghe.com", "https://ACME.ghe.com/", " acme.ghe.com "] {
            assert_eq!(hostname(input), "acme.ghe.com");
        }
        assert_eq!(hostname("github.example.org"), "github.example.org");

        let acme = hostname("acme.ghe.com");
        assert!(trusted("https://acme.ghe.com/login/device", &[&acme]));
        assert!(!trusted("https://other.ghe.com/login/device", &[&acme]));
        assert!(!trusted("https://github.com/login/device", &[&acme]));
        let example = hostname("https://github.example.org");
        assert!(trusted(
            "https://github.example.org/login/device",
            &[&example]
        ));
    }
}
