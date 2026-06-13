use aho_corasick::AhoCorasick;

pub(super) struct Automata {
    pub(super) ci: Option<(AhoCorasick, Vec<(String, i64)>)>,
    pub(super) cs: Option<(AhoCorasick, Vec<(String, i64)>)>,
}
