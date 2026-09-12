use std::collections::HashMap;

const DIRECT_MATCH_MAX_PATTERNS: usize = 8;

#[derive(Debug, Default)]
struct TrieNode {
    children: HashMap<u8, usize>,
    terminal: bool,
}

#[derive(Debug, Default)]
pub(crate) struct ReverseTrie {
    nodes: Vec<TrieNode>,
}

impl ReverseTrie {
    /// Builds a reverse trie from literal suffix patterns.
    ///
    /// Patterns must be sorted from shortest to longest before this
    /// function is called. This allows insertion to skip longer suffixes
    /// that are already covered by a shorter suffix.
    ///
    /// For example, if `ing` has already been inserted, inserting
    /// `something` is unnecessary because every string ending with
    /// `something` also ends with `ing`.
    fn new(patterns: &[String]) -> Self {
        let mut trie = Self {
            nodes: vec![TrieNode::default()],
        };

        for pattern in patterns {
            trie.insert(pattern);
        }

        trie
    }

    /// Inserts a suffix into the trie in reverse byte order.
    ///
    /// If an existing terminal node is encountered while inserting,
    /// the new pattern is redundant and insertion stops immediately.
    fn insert(&mut self, pattern: &str) {
        let mut node_idx = 0;

        for byte in pattern.bytes().rev() {
            if self.nodes[node_idx].terminal {
                return;
            }

            let next_idx = if let Some(&idx) = self.nodes[node_idx].children.get(&byte) {
                idx
            } else {
                let idx = self.nodes.len();

                self.nodes.push(TrieNode::default());

                self.nodes[node_idx].children.insert(byte, idx);

                idx
            };

            node_idx = next_idx;
        }

        self.nodes[node_idx].terminal = true;

        // Any longer suffixes below this node are now redundant.
        self.nodes[node_idx].children.clear();
    }

    /// Returns true if `value` ends with any suffix stored in the trie.
    ///
    /// The input string is traversed backward by UTF-8 byte without
    /// allocating or reversing the string.
    #[inline]
    fn is_match(&self, value: &str) -> bool {
        let mut node_idx = 0;

        for byte in value.bytes().rev() {
            let Some(&next_idx) = self.nodes[node_idx].children.get(&byte) else {
                return false;
            };

            node_idx = next_idx;

            if self.nodes[node_idx].terminal {
                return true;
            }
        }

        self.nodes[node_idx].terminal
    }
}

#[derive(Debug)]
pub(crate) enum SuffixMatcher {
    /// No patterns were supplied.
    Empty,

    /// An empty string was supplied as a pattern.
    ///
    /// Every non-null string ends with the empty string.
    Always,

    /// A single suffix can use Rust's optimized `str::ends_with`.
    Single(String),

    /// Small pattern collections are checked directly.
    ///
    /// For a small number of patterns, repeated `ends_with` calls can
    /// be faster than traversing a more complex matcher.
    Small(Vec<String>),

    /// Larger pattern collections use a reverse trie.
    Trie(ReverseTrie),
}

impl SuffixMatcher {
    /// Creates an optimized matcher for a collection of literal suffixes.
    ///
    /// Patterns are:
    ///
    /// - sorted by length, shortest first;
    /// - deduplicated;
    /// - dispatched to an implementation appropriate for the number
    ///   of patterns.
    ///
    /// Sorting shortest first is important for trie construction. Once a
    /// shorter suffix is terminal, any longer suffix ending with it cannot
    /// change the Boolean result and is therefore ignored.
    pub fn new(mut patterns: Vec<String>) -> Self {
        if patterns.iter().any(String::is_empty) {
            return Self::Always;
        }

        patterns.sort_unstable_by_key(String::len);

        match patterns.len() {
            0 => Self::Empty,

            1 => Self::Single(patterns.into_iter().next().unwrap()),

            2..=DIRECT_MATCH_MAX_PATTERNS => Self::Small(patterns),

            _ => Self::Trie(ReverseTrie::new(&patterns)),
        }
    }

    /// Returns true if `value` ends with any configured suffix.
    #[inline]
    pub fn is_match(&self, value: &str) -> bool {
        match self {
            Self::Empty => false,

            Self::Always => true,

            Self::Single(pattern) => value.ends_with(pattern),

            Self::Small(patterns) => patterns.iter().any(|pattern| value.ends_with(pattern)),

            Self::Trie(trie) => trie.is_match(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_patterns_do_not_match() {
        let matcher = SuffixMatcher::new(vec![]);

        assert!(!matcher.is_match("testing"));
        assert!(!matcher.is_match(""));
    }

    #[test]
    fn empty_pattern_matches_everything() {
        let matcher = SuffixMatcher::new(vec!["ing".to_string(), "".to_string()]);

        assert!(matcher.is_match("testing"));
        assert!(matcher.is_match("anything"));
        assert!(matcher.is_match(""));
    }

    #[test]
    fn single_pattern_matches_suffix() {
        let matcher = SuffixMatcher::new(vec!["ing".to_string()]);

        assert!(matcher.is_match("testing"));
        assert!(matcher.is_match("ing"));
        assert!(!matcher.is_match("test"));
        assert!(!matcher.is_match("ings"));
    }

    #[test]
    fn small_pattern_set_matches_any_suffix() {
        let matcher =
            SuffixMatcher::new(vec!["ing".to_string(), "ed".to_string(), "ly".to_string()]);

        assert!(matcher.is_match("running"));
        assert!(matcher.is_match("walked"));
        assert!(matcher.is_match("quickly"));
        assert!(!matcher.is_match("walk"));
    }

    #[test]
    fn matching_is_case_sensitive() {
        let matcher = SuffixMatcher::new(vec!["ing".to_string()]);

        assert!(matcher.is_match("testing"));
        assert!(!matcher.is_match("TESTING"));
    }

    #[test]
    fn patterns_are_literal() {
        let matcher = SuffixMatcher::new(vec![".b".to_string()]);

        assert!(matcher.is_match("a.b"));
        assert!(!matcher.is_match("axb"));
    }

    #[test]
    fn duplicate_patterns_are_supported() {
        let matcher = SuffixMatcher::new(vec![
            "ing".to_string(),
            "ing".to_string(),
            "ing".to_string(),
        ]);

        assert!(matcher.is_match("testing"));
        assert!(!matcher.is_match("tested"));
    }

    #[test]
    fn reverse_trie_matches_large_pattern_set() {
        let matcher = SuffixMatcher::new(vec![
            "ing".to_string(),
            "ed".to_string(),
            "ly".to_string(),
            "tion".to_string(),
            "ment".to_string(),
            "ness".to_string(),
            "able".to_string(),
            "ous".to_string(),
            "ive".to_string(),
        ]);

        assert!(matcher.is_match("testing"));
        assert!(matcher.is_match("completed"));
        assert!(matcher.is_match("quickly"));
        assert!(matcher.is_match("condition"));
        assert!(matcher.is_match("movement"));
        assert!(!matcher.is_match("example"));
    }

    #[test]
    fn reverse_trie_handles_overlapping_suffixes() {
        let matcher = SuffixMatcher::new(vec![
            "ing".to_string(),
            "thing".to_string(),
            "something".to_string(),
            "testing".to_string(),
            "running".to_string(),
            "walking".to_string(),
            "coding".to_string(),
            "reading".to_string(),
            "writing".to_string(),
        ]);

        assert!(matcher.is_match("thing"));
        assert!(matcher.is_match("something"));
        assert!(matcher.is_match("anything"));
        assert!(matcher.is_match("running"));
        assert!(!matcher.is_match("thingy"));
    }

    #[test]
    fn reverse_trie_handles_unicode() {
        let matcher = SuffixMatcher::new(vec![
            "é".to_string(),
            "ñ".to_string(),
            "β".to_string(),
            "界".to_string(),
            "xyz1".to_string(),
            "xyz2".to_string(),
            "xyz3".to_string(),
            "xyz4".to_string(),
            "xyz5".to_string(),
        ]);

        assert!(matcher.is_match("café"));
        assert!(matcher.is_match("españ"));
        assert!(matcher.is_match("αβ"));
        assert!(matcher.is_match("世界"));
        assert!(!matcher.is_match("cafe"));
    }
}
