use regex::{Regex, RegexBuilder};

use super::span::Span;

pub struct Concept {
    regex: Regex,
}

impl Concept {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        let regex = RegexBuilder::new(pattern).case_insensitive(true).build()?;

        Ok(Self { regex })
    }

    pub fn find_iter<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Span> + 'a {
        self.regex
            .find_iter(text)
            .map(|finding| Span::new(finding.start(), finding.end()))
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub fn count(&self, text: &str) -> usize {
        self.find_iter(text).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_exact_span() {
        let concept = Concept::new(r"\bpneumonia\b").unwrap();

        let text = "Patient has pneumonia.";

        let findings: Vec<Span> = concept.find_iter(text).collect();

        assert_eq!(findings, vec![Span::new(12, 21,),],);
    }

    #[test]
    fn matches_ignore_case() {
        let concept = Concept::new(r"\bpneumonia\b").unwrap();

        assert!(concept.is_match("PNEUMONIA",));
    }

    #[test]
    fn counts_matches() {
        let concept = Concept::new(r"\bpneumonia\b").unwrap();

        assert_eq!(concept.count("Pneumonia and pneumonia.",), 2,);
    }

    #[test]
    fn invalid_regex_returns_error() {
        assert!(Concept::new(r"[invalid",).is_err());
    }
}
