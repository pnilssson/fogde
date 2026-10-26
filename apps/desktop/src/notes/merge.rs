#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

pub fn diff(before: &str, after: &str) -> Option<Change> {
    if before == after {
        return None;
    }

    let start: usize = before
        .char_indices()
        .zip(after.chars())
        .take_while(|((_, a), b)| a == b)
        .last()
        .map_or(0, |((index, char), _)| index + char.len_utf8());
    let end: usize = before[start..]
        .chars()
        .rev()
        .zip(after[start..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(char, _)| char.len_utf8())
        .sum();

    Some(Change {
        from: start,
        to: before.len() - end,
        insert: after[start..after.len() - end].to_owned(),
    })
}

pub fn merge(base: &str, mine: &str, theirs: &str) -> Option<String> {
    let Some(ours) = diff(base, mine).filter(|_| mine != theirs) else {
        return Some(theirs.to_owned());
    };
    let Some(their) = diff(base, theirs) else {
        return Some(mine.to_owned());
    };

    let (first, second) = if ours.from <= their.from {
        (ours, their)
    } else {
        (their, ours)
    };
    if first.to > second.from || first.from == second.from {
        return None;
    }

    Some(format!(
        "{}{}{}{}{}",
        &base[..first.from],
        first.insert,
        &base[first.to..second.from],
        second.insert,
        &base[second.to..]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "# Plan\n\n- login\n- signup\n";

    #[test]
    fn finds_the_one_changed_range_between_two_texts() {
        assert_eq!(diff(BASE, BASE), None);
        assert_eq!(
            diff(BASE, "# Plan\n\n- login flow\n- signup\n"),
            Some(Change {
                from: 15,
                to: 15,
                insert: " flow".into()
            })
        );
        assert_eq!(
            diff("aaa", "aa"),
            Some(Change {
                from: 2,
                to: 3,
                insert: String::new()
            })
        );
    }

    #[test]
    fn keeps_both_sides_when_an_agent_and_the_user_edit_different_parts() {
        let mine = "# Plan for login\n\n- login\n- signup\n";
        let theirs = format!("{BASE}- reset password\n");
        let both = "# Plan for login\n\n- login\n- signup\n- reset password\n";

        assert_eq!(merge(BASE, mine, &theirs).as_deref(), Some(both));
        assert_eq!(merge(BASE, &theirs, mine).as_deref(), Some(both));
    }

    #[test]
    fn takes_the_other_side_when_only_one_side_changed() {
        let theirs = format!("{BASE}- reset password\n");

        assert_eq!(merge(BASE, BASE, &theirs).as_deref(), Some(theirs.as_str()));
        assert_eq!(merge(BASE, &theirs, BASE).as_deref(), Some(theirs.as_str()));
        assert_eq!(
            merge(BASE, &theirs, &theirs).as_deref(),
            Some(theirs.as_str())
        );
    }

    #[test]
    fn gives_up_when_both_sides_change_the_same_text() {
        let mine = "# Plan\n\n- login with email\n- signup\n";
        let theirs = "# Plan\n\n- login with SSO\n- signup\n";

        assert_eq!(merge(BASE, mine, theirs), None);
    }

    #[test]
    fn keeps_windows_line_endings_untouched() {
        let windows = "# Plan\r\n\r\n- login\r\n";
        let mine = "# Plan\r\n\r\n- login\r\n- signup\r\n";
        let theirs = "# Plan for login\r\n\r\n- login\r\n";

        assert_eq!(
            merge(windows, mine, theirs).as_deref(),
            Some("# Plan for login\r\n\r\n- login\r\n- signup\r\n")
        );
    }

    #[test]
    fn handles_text_beyond_ascii() {
        assert_eq!(
            diff("café 👍", "cafés 👍"),
            Some(Change {
                from: 5,
                to: 5,
                insert: "s".into()
            })
        );
    }
}
