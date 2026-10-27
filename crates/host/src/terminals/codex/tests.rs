use std::fs;

use super::*;

#[tokio::test]
async fn installs_the_launcher_and_replaces_an_older_one() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = dir.path().join("codex.exe");
    let installed = dir.path().join("shell").join("bin").join("codex.exe");

    fs::write(&launcher, "first").unwrap();
    let wrap = prepare(&dir.path().join("shell"), Some(&launcher)).await;
    fs::write(&launcher, "second").unwrap();
    prepare(&dir.path().join("shell"), Some(&launcher)).await;

    assert_eq!(wrap.bin.as_deref(), installed.parent());
    assert_eq!(fs::read_to_string(&installed).unwrap(), "second");
}

#[tokio::test]
async fn installs_the_launcher_once_it_has_been_built() {
    let dir = tempfile::tempdir().unwrap();
    let launcher = dir.path().join("codex.exe");

    let before = prepare(&dir.path().join("shell"), Some(&launcher)).await;
    fs::write(&launcher, "built").unwrap();
    let after = prepare(&dir.path().join("shell"), Some(&launcher)).await;

    assert!(before.bin.is_none());
    assert!(after.bin.is_some());
}

#[tokio::test]
async fn adds_no_launcher_where_none_is_shipped() {
    let dir = tempfile::tempdir().unwrap();

    let wrap = prepare(dir.path(), None).await;

    assert!(wrap.bin.is_none());
    assert!(wrap.vars.contains_key("BASH_FUNC_codex%%"));
}

#[cfg(unix)]
mod shells {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::process::Command;

    use super::super::{prepare, unix};

    struct Pane {
        root: tempfile::TempDir,
        env: BTreeMap<String, String>,
    }

    impl Pane {
        async fn new(original: &[(&str, &str)]) -> Self {
            let root = tempfile::tempdir().unwrap();
            let bin = root.path().join("bin");
            fs::create_dir_all(&bin).unwrap();
            fs::create_dir_all(root.path().join("home")).unwrap();
            fs::write(bin.join("codex"), "#!/bin/sh\necho \"$@\"\n").unwrap();
            fs::set_permissions(bin.join("codex"), fs::Permissions::from_mode(0o755)).unwrap();

            let shell = root.path().join("shell");
            let mut env = prepare(&shell, None).await.vars;
            let original: BTreeMap<String, String> = original
                .iter()
                .map(|&(name, value)| (name.to_owned(), value.to_owned()))
                .collect();
            unix::prepare(&shell, &mut env, |name| original.get(name).cloned())
                .await
                .unwrap();

            Self { root, env }
        }

        fn home(&self) -> std::path::PathBuf {
            self.root.path().join("home")
        }

        fn run(&self, shell: &str, args: &[&str]) -> Option<String> {
            let path = format!("{}:/usr/bin:/bin", self.root.path().join("bin").display());
            let output = Command::new(shell)
                .args(args)
                .env_clear()
                .envs(&self.env)
                .env("HOME", self.home())
                .env("PATH", path)
                .output()
                .ok()?;

            Some(String::from_utf8_lossy(&output.stdout).into_owned())
        }
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[tokio::test]
    async fn wraps_codex_in_bash_and_its_subshells() {
        let pane = Pane::new(&[]).await;

        let printed = pane
            .run(
                "bash",
                &["-c", "codex fix; codex agents; bash -c 'codex nested'"],
            )
            .unwrap();

        assert_eq!(printed, "--no-daemon fix\nagents\n--no-daemon nested\n");
    }

    #[tokio::test]
    async fn wraps_codex_in_zsh_and_loads_the_users_own_files() {
        let pane = Pane::new(&[]).await;
        write(&pane.home().join(".zshenv"), "export FROM_ZSHENV=1\n");
        write(&pane.home().join(".zshrc"), "export FROM_ZSHRC=1\n");

        let Some(printed) = pane.run(
            "zsh",
            &[
                "-i",
                "-c",
                "codex fix; echo $FROM_ZSHENV $FROM_ZSHRC ${ZDOTDIR-unset}",
            ],
        ) else {
            return;
        };

        assert_eq!(printed, "--no-daemon fix\n1 1 unset\n");
    }

    #[tokio::test]
    async fn restores_the_users_own_zsh_folder() {
        let dots = tempfile::tempdir().unwrap();
        let dots = dots.path().display().to_string();
        let pane = Pane::new(&[("ZDOTDIR", &dots)]).await;
        write(&Path::new(&dots).join(".zshrc"), "export FROM_ZSHRC=1\n");

        let Some(printed) = pane.run("zsh", &["-i", "-c", "codex fix; echo $FROM_ZSHRC $ZDOTDIR"])
        else {
            return;
        };

        assert_eq!(printed, format!("--no-daemon fix\n1 {dots}\n"));
    }

    #[tokio::test]
    async fn wraps_codex_in_fish_and_restores_the_data_folders() {
        let unset = Pane::new(&[]).await;
        let set = Pane::new(&[("XDG_DATA_DIRS", "/opt/share")]).await;
        let script = "codex fix; codex --remote=ws://localhost:9999; echo \"[$XDG_DATA_DIRS]\"";

        let (Some(unset), Some(set)) = (
            unset.run("fish", &["-c", script]),
            set.run("fish", &["-c", script]),
        ) else {
            return;
        };

        assert_eq!(unset, "--no-daemon fix\n--remote=ws://localhost:9999\n[]\n");
        assert_eq!(
            set,
            "--no-daemon fix\n--remote=ws://localhost:9999\n[/opt/share]\n"
        );
    }
}
