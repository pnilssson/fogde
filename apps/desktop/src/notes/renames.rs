pub fn moved(path: &str, from: &str, to: &str) -> Option<String> {
    let rest = path.strip_prefix(from)?;
    (rest.is_empty() || rest.starts_with(['/', '\\'])).then(|| format!("{to}{rest}"))
}

#[cfg(test)]
mod tests {
    use super::moved;

    #[test]
    fn follows_a_note_when_it_or_a_folder_above_it_is_renamed() {
        let plan = r"C:\notes\app\plan.md";

        assert_eq!(
            moved(plan, plan, r"C:\notes\app\login.md").as_deref(),
            Some(r"C:\notes\app\login.md")
        );
        assert_eq!(
            moved(plan, r"C:\notes\app", r"C:\notes\web").as_deref(),
            Some(r"C:\notes\web\plan.md")
        );
        assert_eq!(
            moved("/notes/app/plan.md", "/notes/app", "/notes/web").as_deref(),
            Some("/notes/web/plan.md")
        );
        assert_eq!(
            moved(r"C:\notes\apps\plan.md", r"C:\notes\app", r"C:\notes\web"),
            None
        );
    }
}
