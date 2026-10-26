use super::{Line, Widget, lines};
use crate::notes::live::parse::{Kind, Mark, parse};

fn preview(text: &str) -> Vec<Line> {
    lines(text, &parse(text), &(usize::MAX..usize::MAX))
}

fn editing(text: &str, line: usize) -> Vec<Line> {
    lines(text, &parse(text), &(line..line + 1))
}

fn marked(line: &Line, mark: &Mark) -> Vec<String> {
    line.marks
        .iter()
        .filter(|(_, found)| found == mark)
        .map(|(range, _)| line.text[range.clone()].to_owned())
        .collect()
}

#[test]
fn hides_heading_marks_except_while_editing() {
    let text = "## Plan\nbody";
    let shown = preview(text);
    assert_eq!(shown[0].text, "Plan");
    assert_eq!(shown[0].kind, Kind::Heading(2));
    assert_eq!(shown[0].source(0), 3);

    let raw = editing(text, 0);
    assert_eq!(raw[0].text, "## Plan");
    assert_eq!(marked(&raw[0], &Mark::Muted), ["## "]);
}

#[test]
fn styles_inline_marks_with_their_markup_hidden() {
    let shown = preview("Some **bold**, *soft*, ~~gone~~ and `code`.");
    let line = &shown[0];
    assert_eq!(line.text, "Some bold, soft, gone and code.");
    assert_eq!(marked(line, &Mark::Strong), ["bold"]);
    assert_eq!(marked(line, &Mark::Emphasis), ["soft"]);
    assert_eq!(marked(line, &Mark::Strike), ["gone"]);
    assert_eq!(marked(line, &Mark::Code), ["code"]);
    assert_eq!(line.source(5), 7);
}

#[test]
fn links_show_only_their_text_and_wikilinks_only_their_label() {
    let shown = preview(
        "See [the docs](https://example.com) and [[plans/login|login plan]] or https://example.org",
    );
    let line = &shown[0];
    assert_eq!(
        line.text,
        "See the docs and login plan or https://example.org"
    );
    assert_eq!(
        marked(line, &Mark::Link("https://example.com".into())),
        ["the docs"]
    );
    assert_eq!(
        marked(line, &Mark::Wikilink("plans/login".into())),
        ["login plan"]
    );
    assert_eq!(
        marked(line, &Mark::Link("https://example.org".into())),
        ["https://example.org"]
    );
}

#[test]
fn bullets_and_tasks_become_widgets() {
    let text = "- one\n  - [x] done\n- [ ] todo\n1. first";
    let shown = preview(text);
    assert_eq!(
        (shown[0].widget.clone(), shown[0].text.as_str()),
        (Widget::Bullet, "one")
    );
    assert_eq!(
        shown[1].widget,
        Widget::Task {
            checked: true,
            at: 10
        }
    );
    assert_eq!(shown[1].indent, 2);
    assert_eq!(shown[1].text, "done");
    assert_eq!(
        shown[2].widget,
        Widget::Task {
            checked: false,
            at: 21
        }
    );
    assert_eq!(shown[3].widget, Widget::None);
    assert_eq!(shown[3].text, "1. first");

    let raw = editing(text, 2);
    assert_eq!(
        (raw[2].widget.clone(), raw[2].text.as_str()),
        (Widget::None, "- [ ] todo")
    );
}

#[test]
fn rules_quotes_and_code_keep_their_lines() {
    let text = "> quoted\n\n---\n\n```\n**raw**\n```";
    let shown = preview(text);
    assert_eq!(shown[0].text, "quoted");
    assert_eq!(shown[0].kind, Kind::Quote);
    assert_eq!(shown[2].widget, Widget::Rule);
    assert_eq!(shown[5].kind, Kind::Code);
    assert_eq!(shown[5].text, "**raw**");
    assert_eq!(shown[4].text, "```");
}

#[test]
fn setext_headings_hide_their_underline() {
    let shown = preview("Title\n=====\nbody");
    assert_eq!(shown[0].kind, Kind::Heading(1));
    assert_eq!(shown[1].text, "");
    assert_eq!(shown[2].kind, Kind::Body);
}

#[test]
fn offsets_count_bytes_in_non_ascii_text() {
    let text = "Héllo **wörld** [[Möte]]";
    let line = &preview(text)[0];
    assert_eq!(line.text, "Héllo wörld Möte");
    assert_eq!(marked(line, &Mark::Strong), ["wörld"]);
    assert_eq!(
        &text[line.source(line.text.len() - "Möte".len())..],
        "Möte]]"
    );
}
