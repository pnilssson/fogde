use super::*;

const WORKING: &str = include_str!("../fixtures/working.diff");

fn found<'a>(chunks: &'a [Chunk], path: &str) -> &'a Chunk {
    chunks
        .iter()
        .find(|chunk| chunk.path == path)
        .unwrap_or_else(|| panic!("no chunk for {path}"))
}

#[test]
fn splits_a_real_diff_into_one_chunk_per_file() {
    let chunks = split(WORKING);

    let paths: Vec<&str> = chunks.iter().map(|chunk| chunk.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "added.txt",
            "app.txt",
            "gone.txt",
            "logo.png",
            "new.txt",
            "spaced name.txt",
            "tab\tname.txt",
            "win.txt"
        ]
    );
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.patch.starts_with("diff --git "))
    );
    assert!(chunks[0].patch.ends_with("+fresh\n"));
}

#[test]
fn reads_each_kind_of_change_with_its_counts() {
    let chunks = split(WORKING);

    let added = found(&chunks, "added.txt");
    assert_eq!(
        (added.change, added.added, added.removed),
        (Change::Added, 1, 0)
    );

    let app = found(&chunks, "app.txt");
    assert_eq!(
        (app.change, app.added, app.removed),
        (Change::Modified, 1, 1)
    );

    let gone = found(&chunks, "gone.txt");
    assert_eq!(
        (gone.change, gone.added, gone.removed),
        (Change::Deleted, 0, 1)
    );

    let logo = found(&chunks, "logo.png");
    assert!(logo.binary);
    assert_eq!((logo.added, logo.removed), (0, 0));

    let renamed = found(&chunks, "new.txt");
    assert_eq!(renamed.change, Change::Renamed);
    assert_eq!(renamed.from.as_deref(), Some("old.txt"));
    assert_eq!((renamed.added, renamed.removed), (1, 1));
}

#[test]
fn reads_paths_git_quotes_or_ends_with_a_tab() {
    let chunks = split(WORKING);

    let spaced = found(&chunks, "spaced name.txt");
    assert_eq!((spaced.added, spaced.removed), (1, 1));
    assert!(spaced.patch.contains("\\ No newline at end of file"));

    assert_eq!(unquote(r#""r\303\251sum\303\251.txt""#), "résumé.txt");
    assert_eq!(unquote(r#""say \"hi\".txt""#), "say \"hi\".txt");
    assert_eq!(unquote("plain.txt"), "plain.txt");
}

#[test]
fn takes_the_path_from_the_header_when_there_are_no_file_lines() {
    let chunks = split(
        "diff --git a/spaced name.bin b/spaced name.bin\nnew file mode 100644\nindex 0000000..6bf43ff\nBinary files /dev/null and b/spaced name.bin differ\n",
    );

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].path, "spaced name.bin");
    assert_eq!(chunks[0].change, Change::Added);
    assert!(chunks[0].binary);
}

#[test]
fn writes_an_untracked_file_as_a_new_file_patch() {
    let patch = untracked("notes/todo.md", b"one\r\ntwo");

    assert_eq!(
        patch,
        "diff --git a/notes/todo.md b/notes/todo.md\nnew file mode 100644\n--- /dev/null\n+++ b/notes/todo.md\n@@ -0,0 +1,2 @@\n+one\r\n+two\n\\ No newline at end of file\n"
    );

    let chunks = split(&patch);
    assert_eq!(chunks[0].path, "notes/todo.md");
    assert_eq!((chunks[0].added, chunks[0].removed), (2, 0));
    assert_eq!(untracked("empty.txt", b"").lines().count(), 4);
}
