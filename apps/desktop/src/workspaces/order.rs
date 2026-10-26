#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Above,
    Below,
}

pub fn arranged(items: &[String], moving: &str, before: Option<&str>) -> Vec<String> {
    let mut rest: Vec<String> = items
        .iter()
        .filter(|item| *item != moving)
        .cloned()
        .collect();
    let at = before
        .and_then(|before| rest.iter().position(|item| item == before))
        .unwrap_or(rest.len());
    rest.insert(at, moving.to_owned());
    rest
}

pub fn before_of(siblings: &[String], moving: &str, target: &str, edge: Edge) -> Option<String> {
    let rest: Vec<&String> = siblings.iter().filter(|id| *id != moving).collect();
    let at = rest.iter().position(|id| *id == target)? + usize::from(edge == Edge::Below);
    rest.get(at).map(|id| (*id).clone())
}

pub fn changes(siblings: &[String], moving: &str, before: Option<&str>) -> bool {
    arranged(siblings, moving, before) != siblings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn siblings() -> Vec<String> {
        ["a", "b", "c", "d"].map(String::from).to_vec()
    }

    fn drop(moving: &str, target: &str, edge: Edge) -> Vec<String> {
        let siblings = siblings();
        let before = before_of(&siblings, moving, target, edge);
        arranged(&siblings, moving, before.as_deref())
    }

    #[test]
    fn dropping_below_the_row_above_leaves_the_order_unchanged() {
        assert_eq!(drop("c", "b", Edge::Below), siblings());
        let before = before_of(&siblings(), "c", "b", Edge::Below);
        assert!(!changes(&siblings(), "c", before.as_deref()));
    }

    #[test]
    fn dropping_above_the_row_below_leaves_the_order_unchanged() {
        let before = before_of(&siblings(), "b", "c", Edge::Above);
        assert!(!changes(&siblings(), "b", before.as_deref()));
    }

    #[test]
    fn dropping_below_a_row_puts_the_terminal_after_it() {
        assert_eq!(drop("a", "c", Edge::Below), ["b", "c", "a", "d"]);
    }

    #[test]
    fn dropping_above_a_row_puts_the_terminal_before_it() {
        assert_eq!(drop("d", "b", Edge::Above), ["a", "d", "b", "c"]);
    }

    #[test]
    fn dropping_below_the_last_row_puts_the_terminal_last() {
        assert_eq!(drop("a", "d", Edge::Below), ["b", "c", "d", "a"]);
    }
}
