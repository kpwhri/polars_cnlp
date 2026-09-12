use super::span::Span;

pub fn tokenize(text: &str) -> Vec<Span> {
    let mut tokens = Vec::new();

    let mut token_start = None;

    for (index, character) in text.char_indices() {
        if character.is_alphanumeric() {
            if token_start.is_none() {
                token_start = Some(index);
            }
        } else if let Some(start) = token_start.take() {
            tokens.push(Span::new(start, index));
        }
    }

    if let Some(start) = token_start {
        tokens.push(Span::new(start, text.len()));
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_text<'a>(text: &'a str, tokens: &[Span]) -> Vec<&'a str> {
        tokens
            .iter()
            .map(|token| &text[token.start..token.end])
            .collect()
    }

    #[test]
    fn tokenizes_words() {
        let text = "No evidence of pneumonia.";

        let tokens = tokenize(text);

        assert_eq!(
            token_text(text, &tokens,),
            vec!["No", "evidence", "of", "pneumonia",],
        );
    }

    #[test]
    fn punctuation_separates_tokens() {
        let text = "rule-out COVID-19";

        let tokens = tokenize(text);

        assert_eq!(
            token_text(text, &tokens,),
            vec!["rule", "out", "COVID", "19",],
        );
    }

    #[test]
    fn apostrophe_separates_tokens() {
        let text = "patient's pneumonia";

        let tokens = tokenize(text);

        assert_eq!(
            token_text(text, &tokens,),
            vec!["patient", "s", "pneumonia",],
        );
    }

    #[test]
    fn empty_text_has_no_tokens() {
        assert!(tokenize("").is_empty());
    }
}
