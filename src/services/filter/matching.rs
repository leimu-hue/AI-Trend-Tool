use aho_corasick::AhoCorasickBuilder;

use crate::models::article::Article;
use crate::models::keyword::Keyword;

use super::traits::{ArticleMatches, KeywordMatcher};
use super::types::Automata;

/// Default keyword matcher using Aho-Corasick multi-pattern automaton.
pub(super) struct AhoCorasickMatcher {
    automata: Automata,
}

impl AhoCorasickMatcher {
    /// Build a new matcher from the given keywords.
    ///
    /// Keywords with `case_sensitive = false` go into a case-insensitive automaton;
    /// keywords with `case_sensitive = true` go into a case-sensitive automaton.
    pub(super) fn new(keywords: &[Keyword]) -> Self {
        let automata = build_automata(keywords);
        Self { automata }
    }
}

impl KeywordMatcher for AhoCorasickMatcher {
    fn match_batch(&self, articles: &[Article]) -> Vec<ArticleMatches> {
        let mut results = Vec::new();

        for article in articles {
            let text = format!("{} {}", article.title, article.summary);
            let mut matched_keyword_ids = Vec::new();

            if let Some((ref ac, ref entries)) = self.automata.ci {
                for mat in ac.find_iter(&text) {
                    matched_keyword_ids.push(entries[mat.pattern()].1);
                }
            }

            if let Some((ref ac, ref entries)) = self.automata.cs {
                for mat in ac.find_iter(&text) {
                    matched_keyword_ids.push(entries[mat.pattern()].1);
                }
            }

            if !matched_keyword_ids.is_empty() {
                results.push(ArticleMatches {
                    article_id: article.id,
                    matched_keyword_ids,
                });
            }
        }

        results
    }
}

/// Build case-insensitive and case-sensitive Aho-Corasick automata from keywords.
fn build_automata(keywords: &[Keyword]) -> Automata {
    let ci_entries: Vec<(String, i64)> = keywords
        .iter()
        .filter(|k| !k.case_sensitive)
        .map(|k| (k.word.to_lowercase(), k.id))
        .collect();
    let cs_entries: Vec<(String, i64)> = keywords
        .iter()
        .filter(|k| k.case_sensitive)
        .map(|k| (k.word.clone(), k.id))
        .collect();

    let ci = if !ci_entries.is_empty() {
        let patterns: Vec<&str> = ci_entries.iter().map(|(w, _)| w.as_str()).collect();
        Some((
            AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .build(patterns)
                .expect("Failed to build CI automaton"),
            ci_entries,
        ))
    } else {
        None
    };

    let cs = if !cs_entries.is_empty() {
        let patterns: Vec<&str> = cs_entries.iter().map(|(w, _)| w.as_str()).collect();
        Some((
            AhoCorasickBuilder::new()
                .build(patterns)
                .expect("Failed to build CS automaton"),
            cs_entries,
        ))
    } else {
        None
    };

    Automata { ci, cs }
}
