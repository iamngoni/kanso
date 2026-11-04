//! Find: case- and diacritic-insensitive search over the reading-order text.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use super::model::{CharPos, Selection, TextDoc};

/// Folds `c` for matching and passes each resulting character to `push`:
/// compatibility-decomposed (so `ﬁ` matches `fi`), without accents, in
/// lower case. All whitespace is one space.
fn fold_char(c: char, push: &mut impl FnMut(char)) {
    if c.is_whitespace() {
        push(' ');
        return;
    }
    for d in std::iter::once(c).nfkd() {
        if is_combining_mark(d) {
            continue;
        }
        for l in d.to_lowercase() {
            push(l);
        }
    }
}

/// A query as folded characters: no accents, no case, one space for any run
/// of whitespace, no space at either end.
fn fold_query(q: &str) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for c in q.chars() {
        fold_char(c, &mut |f| {
            if f == ' ' && (out.is_empty() || out.last() == Some(&' ')) {
                return;
            }
            out.push(f);
        });
    }
    while out.last() == Some(&' ') {
        out.pop();
    }
    out
}

/// A document's text folded for searching, with each folded character tied
/// back to the glyph it came from. Build it once per set of pages and
/// search it as the user types.
#[derive(Clone, Debug, Default)]
pub struct FindIndex {
    chars: Vec<char>,
    src: Vec<CharPos>,
}

impl FindIndex {
    pub fn new(doc: &TextDoc) -> Self {
        let mut chars: Vec<char> = Vec::new();
        let mut src: Vec<CharPos> = Vec::new();
        for page in doc.pages() {
            for (index, g) in page.glyphs.iter().enumerate() {
                let pos = CharPos {
                    page: page.page,
                    index,
                };
                // Line breaks, page breaks and gaps all read as one space.
                let first = chars.is_empty();
                let mut push = |f: char| {
                    if f == ' ' && (chars.is_empty() || chars.last() == Some(&' ')) {
                        return;
                    }
                    chars.push(f);
                    src.push(pos);
                };
                if !first && (index == 0 || g.sep != super::model::Sep::None) {
                    push(' ');
                }
                fold_char(g.ch, &mut push);
            }
        }
        FindIndex { chars, src }
    }

    /// Every hit, left to right, none overlapping, as selections.
    pub fn hits(&self, query: &str) -> Vec<Selection> {
        self.search(query, usize::MAX)
    }

    /// The `n`th hit (0-based), without looking past it.
    pub fn nth(&self, query: &str, n: usize) -> Option<Selection> {
        self.search(query, n.saturating_add(1)).into_iter().nth(n)
    }

    /// How many hits there are.
    pub fn count(&self, query: &str) -> usize {
        self.hits(query).len()
    }

    fn search(&self, query: &str, limit: usize) -> Vec<Selection> {
        let q = fold_query(query);
        let m = q.len();
        let mut out = Vec::new();
        if m == 0 {
            return out;
        }
        let mut i = 0;
        while i + m <= self.chars.len() && out.len() < limit {
            if self.chars[i] == q[0] && self.chars[i..i + m] == q[..] {
                out.push(Selection {
                    start: self.src[i],
                    end: self.src[i + m - 1],
                });
                i += m;
            } else {
                i += 1;
            }
        }
        out
    }
}

/// Folds a whole string the way the index does, for the tests that compare
/// copied text.
#[cfg(test)]
pub(crate) fn fold_str(s: &str) -> String {
    fold_query(s).into_iter().collect()
}

impl TextDoc {
    /// A search index over the pages now in the document.
    pub fn find_index(&self) -> FindIndex {
        FindIndex::new(self)
    }
}
