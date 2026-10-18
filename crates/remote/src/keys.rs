use crate::Error;
use crate::ssh::Server;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Key {
    pub name: String,
    pub fingerprint: String,
}

impl Server {
    pub(crate) async fn offered(&self) -> Result<Vec<Key>, Error> {
        let output = self.run(&["-v"], "exit").await?;

        Ok(offered(&String::from_utf8_lossy(&output.stderr)))
    }
}

fn offered(log: &str) -> Vec<Key> {
    let mut keys: Vec<Key> = Vec::new();

    for line in log.lines() {
        let Some((_, rest)) = line.split_once("Offering public key: ") else {
            continue;
        };
        let words: Vec<&str> = rest.split_whitespace().collect();
        let Some(at) = words.iter().position(|word| word.starts_with("SHA256:")) else {
            continue;
        };
        let key = Key {
            name: words[..at.saturating_sub(1)].join(" "),
            fingerprint: words[at].to_owned(),
        };
        if !keys
            .iter()
            .any(|known| known.fingerprint == key.fingerprint)
        {
            keys.push(key);
        }
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_offered_keys_once() {
        let log = "debug1: Will attempt key: Server ED25519 SHA256:agentKey agent\r\n\
            debug1: Offering public key: Server ED25519 SHA256:agentKey agent\r\n\
            debug1: Authentications that can continue: publickey,password\r\n\
            debug1: Offering public key: C:\\Users\\Jo Doe/.ssh/id_ed25519 ED25519 SHA256:fileKey\r\n\
            debug1: Offering public key: Server ED25519 SHA256:agentKey agent\r\n";

        assert_eq!(
            offered(log),
            [
                Key {
                    name: "Server".to_owned(),
                    fingerprint: "SHA256:agentKey".to_owned(),
                },
                Key {
                    name: "C:\\Users\\Jo Doe/.ssh/id_ed25519".to_owned(),
                    fingerprint: "SHA256:fileKey".to_owned(),
                },
            ]
        );
        assert_eq!(
            offered("debug1: No more authentication methods to try.\r\n"),
            []
        );
    }
}
