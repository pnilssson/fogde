use super::list::native;
use super::*;

#[test]
fn reads_worktrees_and_skips_unusable_ones() {
    let porcelain = "worktree /repos/app\nHEAD 3f2a\nbranch refs/heads/main\n\n\
        worktree /worktrees/app/login\nHEAD 9b1c\nbranch refs/heads/feature/login\n\n\
        worktree /worktrees/app/gone\nHEAD 77aa\nbranch refs/heads/feature/gone\n\
        prunable gitdir file points to non-existent location\n\n\
        worktree /tmp/review\nHEAD 1d0e\ndetached\n\n";

    assert_eq!(
        parse(porcelain),
        [
            Worktree {
                path: native("/repos/app"),
                branch: Some("main".to_owned()),
                main: true,
            },
            Worktree {
                path: native("/worktrees/app/login"),
                branch: Some("feature/login".to_owned()),
                main: false,
            },
            Worktree {
                path: native("/tmp/review"),
                branch: None,
                main: false,
            },
        ]
    );
}

#[test]
fn reads_windows_line_endings() {
    let porcelain = "worktree C:/Users/dev/app\r\nHEAD 3f2a\r\nbranch refs/heads/main\r\n\r\n";

    assert_eq!(
        parse(porcelain),
        [Worktree {
            path: native("C:/Users/dev/app"),
            branch: Some("main".to_owned()),
            main: true,
        }]
    );
}
