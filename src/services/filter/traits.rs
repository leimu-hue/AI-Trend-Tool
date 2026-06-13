use crate::models::article::Article;

/// Abstract interface for keyword matching strategies.
///
/// Current implementation: `AhoCorasickMatcher` (Aho-Corasick automaton).
/// Future: AI-based semantic matcher can implement this trait.
pub trait KeywordMatcher: Send + Sync {
    /// Match a batch of articles against the matcher's keyword set.
    ///
    /// Returns one `ArticleMatches` per article that had at least one keyword hit.
    /// Articles with zero matches are omitted from the result.
    fn match_batch(&self, articles: &[Article]) -> Vec<ArticleMatches>;
}

/// Per-article keyword match result.
pub struct ArticleMatches {
    pub article_id: i64,
    pub matched_keyword_ids: Vec<i64>,
}
