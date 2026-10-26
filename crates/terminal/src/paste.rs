const START: &str = "\x1b[200~";
const END: &str = "\x1b[201~";

pub fn bracket(text: &str) -> String {
    let mut clean = text.replace("\r\n", "\n").replace('\r', "\n");
    while clean.contains(START) || clean.contains(END) {
        clean = clean.replace(START, "").replace(END, "");
    }

    format!("{START}{clean}{END}")
}

pub fn bracket_path(path: &str) -> String {
    let plain = !path.is_empty()
        && path
            .chars()
            .all(|char| char.is_ascii_alphanumeric() || "_./\\:~-".contains(char));

    if plain {
        bracket(path)
    } else {
        bracket(&format!("\"{path}\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paste_is_one_bracketed_paste_with_plain_newlines() {
        assert_eq!(
            bracket("one\r\ntwo\rthree\nfour"),
            "\x1b[200~one\ntwo\nthree\nfour\x1b[201~"
        );
    }

    #[test]
    fn pasted_text_cannot_end_the_paste_early() {
        assert_eq!(
            bracket("safe\x1b[201~rm -rf ~\r\n\x1b[200~"),
            "\x1b[200~saferm -rf ~\n\x1b[201~"
        );
        assert_eq!(
            bracket("safe\x1b[20\x1b[201~1~rm -rf ~"),
            "\x1b[200~saferm -rf ~\x1b[201~"
        );
    }

    #[test]
    fn a_pasted_path_is_quoted_only_when_a_shell_would_split_it() {
        assert_eq!(
            bracket_path("/notes/plan.md"),
            "\x1b[200~/notes/plan.md\x1b[201~"
        );
        assert_eq!(
            bracket_path("C:\\notes\\Login flow.md"),
            "\x1b[200~\"C:\\notes\\Login flow.md\"\x1b[201~"
        );
        assert_eq!(
            bracket_path("/notes/Q&A;(draft).md"),
            "\x1b[200~\"/notes/Q&A;(draft).md\"\x1b[201~"
        );
    }
}
