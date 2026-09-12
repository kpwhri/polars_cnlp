pub fn contains_target(text: &str, target: &str) -> bool {
    text.contains(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_target() {
        assert!(contains_target("Patient has pneumonia.", "pneumonia",));
    }

    #[test]
    fn target_not_present() {
        assert!(!contains_target("Patient has asthma.", "pneumonia",));
    }
}
