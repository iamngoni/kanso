//! Small rules about a note itself that the apps share.

/// The title a copy of a note is given: the note's, then " (copy)". Pure.
pub fn copy_title(title: &str) -> String {
    format!("{title} (copy)")
}

#[cfg(test)]
mod tests {
    use super::copy_title;

    #[test]
    fn a_copy_says_so() {
        assert_eq!(copy_title("Plan"), "Plan (copy)");
        assert_eq!(copy_title(""), " (copy)");
        assert_eq!(copy_title("Plan (copy)"), "Plan (copy) (copy)");
    }
}
