use super::concept::Concept;

pub fn count(text: &str, concept: &Concept) -> usize {
    concept.count(text)
}

pub fn count_all(text: &str, concepts: &[Concept]) -> Vec<usize> {
    concepts
        .iter()
        .map(|concept| count(text, concept))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn concept(pattern: &str) -> Concept {
        Concept::new(pattern).unwrap()
    }

    #[test]
    fn counts_zero_occurrences() {
        assert_eq!(
            count("Patient has asthma.", &concept(r"\bpneumonia\b",),),
            0,
        );
    }

    #[test]
    fn counts_multiple_occurrences() {
        assert_eq!(
            count(
                "Pneumonia improved. Pneumonia later recurred.",
                &concept(r"\bpneumonia\b",),
            ),
            2,
        );
    }

    #[test]
    fn counts_ignore_case() {
        assert_eq!(
            count(
                "Pneumonia, PNEUMONIA, pneumonia.",
                &concept(r"\bpneumonia\b",),
            ),
            3,
        );
    }

    #[test]
    fn count_all_preserves_concept_order() {
        let concepts = vec![
            concept(r"\bpneumonia\b"),
            concept(r"\basthma\b"),
            concept(r"\banaphylaxis\b"),
        ];

        assert_eq!(
            count_all("Pneumonia. Asthma and asthma.", &concepts,),
            vec![1, 2, 0,],
        );
    }

    #[test]
    fn count_all_empty_concepts_returns_empty_vector() {
        assert!(count_all("Pneumonia.", &[],).is_empty());
    }
}
