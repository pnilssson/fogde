use std::path::Path;

use tokio::fs;

use crate::Error;
use crate::run::{git, git_paths};

const UNFINISHED: [(&str, &str); 5] = [
    ("MERGE_HEAD", "merge"),
    ("rebase-merge", "rebase"),
    ("rebase-apply", "rebase"),
    ("CHERRY_PICK_HEAD", "cherry-pick"),
    ("REVERT_HEAD", "revert"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LastCommit {
    pub sha: String,
    pub subject: String,
    pub date: String,
    pub undoable: bool,
}

pub async fn last(root: &Path) -> Option<LastCommit> {
    let output = git(
        root,
        ["log", "-1", "--format=%H%x00%P%x00%cI%x00%s", "HEAD"],
    )
    .await
    .ok()?;
    let output = String::from_utf8_lossy(&output);
    let mut fields = output.trim_end_matches('\n').splitn(4, '\0');
    let (sha, parents, date, subject) = (
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
    );
    let single = parents.split_whitespace().count() == 1;

    Some(LastCommit {
        sha: sha.to_owned(),
        subject: subject.to_owned(),
        date: date.to_owned(),
        undoable: single && !published(root).await && matches!(unfinished(root).await, Ok(None)),
    })
}

pub async fn undo(root: &Path, sha: &str) -> Result<(), Error> {
    let Some(last) = last(root).await else {
        return Err(Error::Git("there is no commit to undo".to_owned()));
    };
    if last.sha != sha {
        return Err(Error::Git(
            "the branch has a newer commit; refresh and try again".to_owned(),
        ));
    }
    if let Some(operation) = unfinished(root).await? {
        return Err(Error::Git(format!(
            "finish or abort the {operation} in a terminal first"
        )));
    }
    if !last.undoable {
        return Err(Error::Git(
            "only an unpushed commit with one parent can be undone".to_owned(),
        ));
    }

    git(root, ["reset", "--soft", "--quiet", "HEAD~1"])
        .await
        .map(drop)
}

async fn unfinished(root: &Path) -> Result<Option<&'static str>, Error> {
    let paths = git_paths(root, &UNFINISHED.map(|(path, _)| path)).await?;

    for (path, (_, operation)) in paths.into_iter().zip(UNFINISHED) {
        if fs::try_exists(path).await? {
            return Ok(Some(operation));
        }
    }

    Ok(None)
}

async fn published(root: &Path) -> bool {
    let output = git(
        root,
        [
            "for-each-ref",
            "--count=1",
            "--contains=HEAD",
            "--format=%(refname)",
            "refs/remotes",
        ],
    )
    .await;

    match output {
        Ok(output) => !output.is_empty(),
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commit::commit;
    use crate::status::status;
    use crate::testing::{log, repo, write};

    #[tokio::test]
    async fn undoes_the_last_unpushed_commit_and_keeps_its_changes() {
        let dir = tempfile::tempdir().unwrap();
        repo(dir.path()).await;
        write(dir.path(), "app.txt", "one\n").await;
        commit(dir.path(), &["app.txt".to_owned()], "start")
            .await
            .unwrap();
        write(dir.path(), "app.txt", "two\n").await;
        commit(dir.path(), &["app.txt".to_owned()], "Second")
            .await
            .unwrap();

        let second = last(dir.path()).await.unwrap();
        assert_eq!(second.subject, "Second");
        assert!(second.undoable);

        undo(dir.path(), &second.sha).await.unwrap();

        assert_eq!(log(dir.path()).await, ["start"]);
        let files = status(dir.path()).await.unwrap().files;
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "app.txt");
    }

    #[tokio::test]
    async fn refuses_the_first_commit_a_pushed_one_or_a_stale_sha() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        tokio::fs::create_dir(&app).await.unwrap();
        repo(&app).await;
        write(&app, "app.txt", "one\n").await;
        commit(&app, &["app.txt".to_owned()], "start")
            .await
            .unwrap();

        let first = last(&app).await.unwrap();
        assert!(!first.undoable);
        assert!(undo(&app, &first.sha).await.is_err());

        write(&app, "app.txt", "two\n").await;
        commit(&app, &["app.txt".to_owned()], "Second")
            .await
            .unwrap();
        assert!(undo(&app, &first.sha).await.is_err());

        git(
            dir.path(),
            [
                "init",
                "--quiet",
                "--bare",
                "--initial-branch",
                "main",
                "remote.git",
            ],
        )
        .await
        .unwrap();
        let remote = dir.path().join("remote.git");
        git(&app, ["remote", "add", "origin", remote.to_str().unwrap()])
            .await
            .unwrap();
        git(
            &app,
            ["push", "--quiet", "--set-upstream", "origin", "HEAD"],
        )
        .await
        .unwrap();

        let pushed = last(&app).await.unwrap();
        assert!(!pushed.undoable);
        assert!(undo(&app, &pushed.sha).await.is_err());
        assert_eq!(log(&app).await, ["Second", "start"]);
    }

    async fn diverged(root: &Path) {
        repo(root).await;
        write(root, "app.txt", "one\n").await;
        commit(root, &["app.txt".to_owned()], "Start")
            .await
            .unwrap();
        git(root, ["switch", "--quiet", "-c", "feature/login"])
            .await
            .unwrap();
        write(root, "app.txt", "login\n").await;
        commit(root, &["app.txt".to_owned()], "Login")
            .await
            .unwrap();
        git(root, ["switch", "--quiet", "main"]).await.unwrap();
        write(root, "app.txt", "main\n").await;
        commit(root, &["app.txt".to_owned()], "Main").await.unwrap();
    }

    async fn refused(root: &Path, operation: &str) {
        let head = last(root).await.unwrap();
        assert!(!head.undoable);

        let error = undo(root, &head.sha).await.unwrap_err();

        assert!(error.to_string().contains(operation), "{error}");
        assert_eq!(last(root).await.unwrap().sha, head.sha);
    }

    #[tokio::test]
    async fn refuses_during_a_rebase() {
        let dir = tempfile::tempdir().unwrap();
        diverged(dir.path()).await;
        git(dir.path(), ["switch", "--quiet", "feature/login"])
            .await
            .unwrap();
        assert!(
            git(dir.path(), ["rebase", "--quiet", "main"])
                .await
                .is_err()
        );

        refused(dir.path(), "rebase").await;
    }

    #[tokio::test]
    async fn refuses_during_a_merge_in_a_linked_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let app = dir.path().join("app");
        let login = dir.path().join("login");
        tokio::fs::create_dir(&app).await.unwrap();
        diverged(&app).await;
        git(
            &app,
            [
                "worktree",
                "add",
                "--quiet",
                login.to_str().unwrap(),
                "feature/login",
            ],
        )
        .await
        .unwrap();
        assert!(git(&login, ["merge", "--quiet", "main"]).await.is_err());

        refused(&login, "merge").await;
        assert!(last(&app).await.unwrap().undoable);
    }

    #[tokio::test]
    async fn refuses_during_a_cherry_pick() {
        let dir = tempfile::tempdir().unwrap();
        diverged(dir.path()).await;
        assert!(
            git(dir.path(), ["cherry-pick", "feature/login"])
                .await
                .is_err()
        );

        refused(dir.path(), "cherry-pick").await;
    }

    #[tokio::test]
    async fn refuses_during_a_revert() {
        let dir = tempfile::tempdir().unwrap();
        diverged(dir.path()).await;
        write(dir.path(), "app.txt", "later\n").await;
        commit(dir.path(), &["app.txt".to_owned()], "Later")
            .await
            .unwrap();
        assert!(
            git(dir.path(), ["revert", "--no-edit", "HEAD~1"])
                .await
                .is_err()
        );

        refused(dir.path(), "revert").await;
    }
}
