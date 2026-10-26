use super::{Doc, next, previous, word_left, word_right};

fn at(text: &str, caret: usize) -> Doc {
    let mut doc = Doc::new(text.to_owned());
    doc.select(caret, caret);
    doc
}

#[test]
fn moves_over_whole_graphemes() {
    let text = "a👍🏽b";
    assert_eq!(next(text, 1), 9);
    assert_eq!(previous(text, 9), 1);
    assert_eq!(previous(text, 0), 0);
    assert_eq!(next(text, text.len()), text.len());
}

#[test]
fn moves_by_words_and_stops_at_line_breaks() {
    let text = "fix the login\nflow";
    assert_eq!(word_right(text, 0), 3);
    assert_eq!(word_right(text, 3), 7);
    assert_eq!(word_right(text, 13), 14);
    assert_eq!(word_left(text, 13), 8);
    assert_eq!(word_left(text, 14), 13);
    assert_eq!(word_left(text, 8), 4);
}

#[test]
fn enter_continues_lists_quotes_and_tasks() {
    let mut doc = at("- one", 5);
    doc.newline();
    assert_eq!(doc.text, "- one\n- ");

    let mut doc = at("  9. nine", 9);
    doc.newline();
    assert_eq!(doc.text, "  9. nine\n  10. ");

    let mut doc = at("- [x] done", 10);
    doc.newline();
    assert_eq!(doc.text, "- [x] done\n- [ ] ");

    let mut doc = at("> quote", 7);
    doc.newline();
    assert_eq!(doc.text, "> quote\n> ");

    let mut doc = at("plain", 5);
    doc.newline();
    assert_eq!(doc.text, "plain\n");
}

#[test]
fn enter_on_an_empty_item_ends_the_list() {
    let mut doc = at("- one\n- ", 8);
    doc.newline();
    assert_eq!(doc.text, "- one\n");
    assert_eq!(doc.head, 6);
}

#[test]
fn backspace_after_markup_removes_it() {
    let mut doc = at("- [ ] task", 6);
    doc.delete_backward();
    assert_eq!(doc.text, "task");

    let mut doc = at("ab", 2);
    doc.delete_backward();
    assert_eq!(doc.text, "a");
}

#[test]
fn tab_indents_every_selected_line_and_keeps_the_selection() {
    let mut doc = Doc::new("one\ntwo\nthree".to_owned());
    doc.select(1, 6);
    doc.indent();
    assert_eq!(doc.text, "  one\n  two\nthree");
    assert_eq!(doc.selection(), 3..10);

    doc.outdent();
    assert_eq!(doc.text, "one\ntwo\nthree");
    assert_eq!(doc.selection(), 1..6);
}

#[test]
fn outdent_moves_a_caret_inside_the_indent_to_the_line_start() {
    let mut doc = at("    deep", 1);
    doc.outdent();
    assert_eq!(doc.text, "  deep");
    assert_eq!(doc.head, 0);
}

#[test]
fn wrapping_toggles_markers_around_the_selection() {
    let mut doc = Doc::new("make it bold".to_owned());
    doc.select(8, 12);
    doc.wrap("**");
    assert_eq!(doc.text, "make it **bold**");
    assert_eq!(doc.selection(), 10..14);

    doc.wrap("**");
    assert_eq!(doc.text, "make it bold");
    assert_eq!(doc.selection(), 8..12);
}

#[test]
fn toggles_a_task_and_keeps_the_caret() {
    let mut doc = at("- [ ] ship", 10);
    doc.toggle_task(2);
    assert_eq!(doc.text, "- [x] ship");
    doc.toggle_task(2);
    assert_eq!(doc.text, "- [ ] ship");
    assert_eq!(doc.head, 10);
}

#[test]
fn replacing_clamps_to_character_boundaries() {
    let mut doc = Doc::new("héllo".to_owned());
    doc.select(2, 2);
    assert_eq!(doc.head, 1);
    doc.replace(1..3, "e");
    assert_eq!(doc.text, "hello");
}
