use super::boundary::has_hard_boundary;
use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenRange {
    pub start: usize,
    pub end: usize,
}

impl TokenRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

pub fn token_range(span: Span, tokens: &[Span]) -> Option<TokenRange> {
    let start = tokens.iter().position(|token| token.overlaps(span))?;

    let end = tokens.iter().rposition(|token| token.overlaps(span))? + 1;

    Some(TokenRange::new(start, end))
}

pub fn tokens_between(left: TokenRange, right: TokenRange) -> Option<usize> {
    if left.end <= right.start {
        Some(right.start - left.end)
    } else if right.end <= left.start {
        Some(left.start - right.end)
    } else {
        None
    }
}

pub fn sentence_token_range(text: &str, tokens: &[Span], anchor: TokenRange) -> Option<TokenRange> {
    if tokens.is_empty() || anchor.start >= anchor.end || anchor.end > tokens.len() {
        return None;
    }

    let mut start = anchor.start;

    while start > 0 {
        let left = tokens[start - 1];

        let right = tokens[start];

        if has_hard_boundary(text, left.end, right.start) {
            break;
        }

        start -= 1;
    }

    let mut end = anchor.end;

    while end < tokens.len() {
        let left = tokens[end - 1];

        let right = tokens[end];

        if has_hard_boundary(text, left.end, right.start) {
            break;
        }

        end += 1;
    }

    Some(TokenRange::new(start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::tokenizer::tokenize;

    #[test]
    fn maps_single_token_span() {
        let text = "No pneumonia.";

        let tokens = tokenize(text);

        assert_eq!(
            token_range(Span::new(3, 12), &tokens,),
            Some(TokenRange::new(1, 2,),),
        );
    }

    #[test]
    fn maps_multi_token_span() {
        let text = "rule-out pneumonia";

        let tokens = tokenize(text);

        assert_eq!(
            token_range(Span::new(0, 8), &tokens,),
            Some(TokenRange::new(0, 2,),),
        );
    }

    #[test]
    fn adjacent_ranges_have_zero_tokens_between() {
        assert_eq!(
            tokens_between(TokenRange::new(0, 2,), TokenRange::new(2, 3,),),
            Some(0),
        );
    }

    #[test]
    fn one_intervening_token_has_distance_one() {
        assert_eq!(
            tokens_between(TokenRange::new(0, 2,), TokenRange::new(3, 4,),),
            Some(1),
        );
    }

    #[test]
    fn overlapping_ranges_have_no_distance() {
        assert_eq!(
            tokens_between(TokenRange::new(0, 2,), TokenRange::new(1, 3,),),
            None,
        );
    }

    #[test]
    fn finds_first_sentence_range() {
        let text = "No fever. Pneumonia present.";

        let tokens = tokenize(text);

        assert_eq!(
            sentence_token_range(text, &tokens, TokenRange::new(0, 1,),),
            Some(TokenRange::new(0, 2,),),
        );
    }

    #[test]
    fn finds_second_sentence_range() {
        let text = "No fever. Pneumonia present.";

        let tokens = tokenize(text);

        assert_eq!(
            sentence_token_range(text, &tokens, TokenRange::new(2, 3,),),
            Some(TokenRange::new(2, 4,),),
        );
    }
}
