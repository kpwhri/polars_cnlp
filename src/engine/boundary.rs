pub fn has_hard_boundary(text: &str, start: usize, end: usize) -> bool {
    text[start..end].chars().any(is_hard_boundary)
}

fn is_hard_boundary(character: char) -> bool {
    matches!(character, '.' | '?' | '!' | ';' | '\n' | '\r')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_is_hard_boundary() {
        let text = "No fever. Pneumonia.";

        assert!(has_hard_boundary(text, 2, 10,));
    }

    #[test]
    fn newline_is_hard_boundary() {
        let text = "No fever\nPneumonia.";

        assert!(has_hard_boundary(text, 2, 9,));
    }

    #[test]
    fn comma_is_not_hard_boundary() {
        let text = "No fever, pneumonia";

        assert!(!has_hard_boundary(text, 2, 10,));
    }

    #[test]
    fn empty_range_has_no_boundary() {
        assert!(!has_hard_boundary("abc", 1, 1,));
    }
}
