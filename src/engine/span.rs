#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    pub fn between(self, other: Self) -> Option<Self> {
        if self.end <= other.start {
            Some(Self::new(self.end, other.start))
        } else if other.end <= self.start {
            Some(Self::new(other.end, self.start))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_spans_overlap() {
        let left = Span::new(0, 5);

        let right = Span::new(3, 8);

        assert!(left.overlaps(right));
    }

    #[test]
    fn adjacent_spans_do_not_overlap() {
        let left = Span::new(0, 5);

        let right = Span::new(5, 8);

        assert!(!left.overlaps(right));
    }

    #[test]
    fn finds_span_between_forward_spans() {
        let left = Span::new(0, 3);

        let right = Span::new(5, 8);

        assert_eq!(left.between(right), Some(Span::new(3, 5)),);
    }

    #[test]
    fn finds_span_between_reverse_spans() {
        let left = Span::new(5, 8);

        let right = Span::new(0, 3);

        assert_eq!(left.between(right), Some(Span::new(3, 5)),);
    }

    #[test]
    fn overlapping_spans_have_no_between_span() {
        let left = Span::new(0, 5);

        let right = Span::new(3, 8);

        assert_eq!(left.between(right), None,);
    }

    #[test]
    fn adjacent_spans_have_empty_between_span() {
        let left = Span::new(0, 3);

        let right = Span::new(3, 8);

        assert_eq!(left.between(right), Some(Span::new(3, 3)),);
    }
}
