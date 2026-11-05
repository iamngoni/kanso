//! Selection, copy, word and line expansion, and highlight quads.

use std::collections::BTreeMap;
use std::ops::Range;

use super::layout::layout_page;
use super::model::{
    CharPos, HighlightParts, PAGE_BREAK, PageLayout, PageText, Point, Selection, Sep, TextDoc,
};
use crate::model::{MAX_QUADS, MAX_QUOTE_CHARS, Quad};

impl TextDoc {
    /// Lay out `pages`. A page given twice keeps the later one.
    pub fn new<'a>(pages: impl IntoIterator<Item = &'a PageText>) -> Self {
        let mut by_page: BTreeMap<u32, PageLayout> = BTreeMap::new();
        for p in pages {
            by_page.insert(p.page, layout_page(p));
        }
        TextDoc {
            pages: by_page.into_values().collect(),
        }
    }

    /// Add or replace one page, as the worker delivers them.
    pub fn set_page(&mut self, page: &PageText) {
        let layout = layout_page(page);
        match self.pages.binary_search_by_key(&page.page, |p| p.page) {
            Ok(i) => self.pages[i] = layout,
            Err(i) => self.pages.insert(i, layout),
        }
    }

    pub fn pages(&self) -> &[PageLayout] {
        &self.pages
    }

    pub fn page(&self, page: u32) -> Option<&PageLayout> {
        self.pages
            .binary_search_by_key(&page, |p| p.page)
            .ok()
            .map(|i| &self.pages[i])
    }

    /// The character nearest the point, or `None` when the page has no text.
    /// The pointer never has to be exactly on a character: a click in the
    /// margin or between lines picks the closest one.
    pub fn hit(&self, p: Point) -> Option<CharPos> {
        let page = self.page(p.page)?;
        let (w, h) = (page.width.max(1e-9), page.height.max(1e-9));
        let mut best: Option<(usize, f64)> = None;
        for (i, g) in page.glyphs.iter().enumerate() {
            let [x0, y0, x1, y1] = g.quad.0;
            let dx = (x0 - p.x).max(p.x - x1).max(0.0) * w;
            let dy = (y0 - p.y).max(p.y - y1).max(0.0) * h;
            let d = dx * dx + dy * dy;
            // Ties go to the earlier character in reading order.
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        best.map(|(index, _)| CharPos {
            page: p.page,
            index,
        })
    }

    /// The selection a drag from `a` to `b` makes. The order of the two
    /// points does not matter.
    pub fn select(&self, a: Point, b: Point) -> Option<Selection> {
        Some(Selection::between(self.hit(a)?, self.hit(b)?))
    }

    /// Every character of every page.
    pub fn select_all(&self) -> Option<Selection> {
        let first = self.pages.iter().find(|p| !p.glyphs.is_empty())?;
        let last = self.pages.iter().rev().find(|p| !p.glyphs.is_empty())?;
        Some(Selection {
            start: CharPos {
                page: first.page,
                index: 0,
            },
            end: CharPos {
                page: last.page,
                index: last.glyphs.len() - 1,
            },
        })
    }

    fn glyph_ok(&self, pos: CharPos) -> Option<&PageLayout> {
        self.page(pos.page).filter(|p| pos.index < p.glyphs.len())
    }

    /// Double-click: the word at `pos`.
    pub fn word_at(&self, pos: CharPos) -> Option<Selection> {
        let page = self.glyph_ok(pos)?;
        let line = &page.lines[page.glyphs[pos.index].line as usize];
        let ls = line.glyphs.start;
        let le = line.glyphs.end;
        let class = |k: usize| word_class(page, ls, le, k);
        let want = class(pos.index);
        let mut a = pos.index;
        while a > ls && page.glyphs[a].sep == Sep::None && class(a - 1) == want {
            a -= 1;
        }
        let mut b = pos.index;
        while b + 1 < le && page.glyphs[b + 1].sep == Sep::None && class(b + 1) == want {
            b += 1;
        }
        Some(span(pos.page, a..b + 1))
    }

    /// Triple-click: the line at `pos`. A line is one column's line, not the
    /// whole row of a two-column page.
    pub fn line_at(&self, pos: CharPos) -> Option<Selection> {
        let page = self.glyph_ok(pos)?;
        let line = &page.lines[page.glyphs[pos.index].line as usize];
        Some(span(pos.page, line.glyphs.clone()))
    }

    /// The paragraph (block) at `pos`.
    pub fn paragraph_at(&self, pos: CharPos) -> Option<Selection> {
        let page = self.glyph_ok(pos)?;
        let block = &page.blocks[page.glyphs[pos.index].block as usize];
        Some(span(pos.page, block.glyphs.clone()))
    }

    /// The characters a selection covers, per page, as reading-order ranges.
    fn ranges(&self, sel: &Selection) -> Vec<(&PageLayout, Range<usize>)> {
        let mut out = Vec::new();
        for p in &self.pages {
            if p.page < sel.start.page || p.page > sel.end.page || p.glyphs.is_empty() {
                continue;
            }
            let lo = if p.page == sel.start.page {
                sel.start.index
            } else {
                0
            };
            let hi = if p.page == sel.end.page {
                (sel.end.index + 1).min(p.glyphs.len())
            } else {
                p.glyphs.len()
            };
            if lo < hi {
                out.push((p, lo..hi));
            }
        }
        out
    }

    /// The text a selection copies as.
    pub fn text(&self, sel: &Selection) -> String {
        let mut s = String::new();
        let mut first = true;
        for (p, r) in self.ranges(sel) {
            for i in r.clone() {
                let g = &p.glyphs[i];
                if i == r.start {
                    if !first {
                        s.push_str(PAGE_BREAK);
                    }
                } else {
                    s.push_str(g.sep.as_str());
                }
                s.push(g.ch);
            }
            first = false;
        }
        s
    }

    /// One quad per line a selection touches, by page. The quad covers the
    /// selected characters along the line and the whole line across it, so
    /// the marks line up from one line to the next.
    pub fn quads(&self, sel: &Selection) -> BTreeMap<u32, Vec<Quad>> {
        let mut out: BTreeMap<u32, Vec<Quad>> = BTreeMap::new();
        for (p, r) in self.ranges(sel) {
            let mut i = r.start;
            while i < r.end {
                let line = &p.lines[p.glyphs[i].line as usize];
                let end = line.glyphs.end.min(r.end);
                let mut u = p.glyphs[i].quad.0;
                for g in &p.glyphs[i..end] {
                    let q = g.quad.0;
                    u = [
                        u[0].min(q[0]),
                        u[1].min(q[1]),
                        u[2].max(q[2]),
                        u[3].max(q[3]),
                    ];
                }
                let b = line.band.0;
                let q = if line.turns % 2 == 0 {
                    Quad::new(u[0], b[1], u[2], b[3])
                } else {
                    Quad::new(b[0], u[1], b[2], u[3])
                };
                out.entry(p.page).or_default().push(q);
                i = end;
            }
        }
        out
    }

    /// What step 1's highlight stores for a selection: quads within the
    /// 32-quad cap and the quoted text within its cap.
    pub fn highlight_parts(&self, sel: &Selection) -> HighlightParts {
        let mut quads = self.quads(sel);
        shrink_to(&mut quads, MAX_QUADS);
        HighlightParts {
            quads,
            text: self.text(sel).chars().take(MAX_QUOTE_CHARS).collect(),
        }
    }

    /// Resolves a quoted phrase in one laid-out page and returns the exact
    /// quads the reader would store for a selection of those glyphs.
    pub fn find_quote(&self, page: u32, quote: &str) -> Option<HighlightParts> {
        let layout = self.page(page)?;
        let target = folded_words(quote);
        if target.is_empty() {
            return None;
        }
        let mut text = Vec::new();
        let mut owners = Vec::new();
        for (index, glyph) in layout.glyphs.iter().enumerate() {
            for ch in glyph.sep.as_str().chars() {
                if ch.is_whitespace() {
                    if !text.is_empty() && text.last() != Some(&' ') {
                        text.push(' ');
                        owners.push(None);
                    }
                } else {
                    for folded in ch.to_lowercase() {
                        text.push(folded);
                        owners.push(None);
                    }
                }
            }
            for folded in glyph.ch.to_lowercase() {
                text.push(folded);
                owners.push(Some(index));
            }
        }
        let start = text
            .windows(target.len())
            .position(|window| window == target)?;
        let positions: Vec<usize> = owners[start..start + target.len()]
            .iter()
            .flatten()
            .copied()
            .collect();
        let first = *positions.first()?;
        let last = *positions.last()?;
        Some(self.highlight_parts(&Selection {
            start: CharPos { page, index: first },
            end: CharPos { page, index: last },
        }))
    }
}

fn folded_words(text: &str) -> Vec<char> {
    let mut normalized = Vec::new();
    for word in text.split_whitespace() {
        if !normalized.is_empty() {
            normalized.push(' ');
        }
        normalized.extend(word.chars().flat_map(char::to_lowercase));
    }
    normalized
}

impl Selection {
    /// The selection from one position to another, either way round.
    pub fn between(a: CharPos, b: CharPos) -> Self {
        Selection {
            start: a.min(b),
            end: a.max(b),
        }
    }

    /// Whether `pos` is selected.
    pub fn contains(&self, pos: CharPos) -> bool {
        self.start <= pos && pos <= self.end
    }

    /// Extends the selection to `pos` (Shift-click), keeping the end that
    /// is farther from it as the anchor.
    pub fn extend_to(&self, pos: CharPos) -> Selection {
        if pos < self.start {
            Selection {
                start: pos,
                end: self.end,
            }
        } else if pos > self.end {
            Selection {
                start: self.start,
                end: pos,
            }
        } else {
            // Inside: move the nearer end.
            let near_start = pos.page == self.start.page
                && (pos.page != self.end.page
                    || pos.index - self.start.index <= self.end.index - pos.index);
            if near_start {
                Selection {
                    start: pos,
                    end: self.end,
                }
            } else {
                Selection {
                    start: self.start,
                    end: pos,
                }
            }
        }
    }
}

fn span(page: u32, r: Range<usize>) -> Selection {
    Selection {
        start: CharPos {
            page,
            index: r.start,
        },
        end: CharPos {
            page,
            index: r.end - 1,
        },
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Class {
    Word,
    Other,
}

/// Letters, digits and `_` make words; an apostrophe between two of them
/// stays in the word (`don't`). Anything else is its own kind of run.
fn word_class(page: &PageLayout, ls: usize, le: usize, k: usize) -> Class {
    let c = page.glyphs[k].ch;
    if c.is_alphanumeric() || c == '_' {
        return Class::Word;
    }
    if (c == '\'' || c == '\u{2019}') && k > ls && k + 1 < le {
        let joined = |j: usize| {
            let g = &page.glyphs[j];
            g.ch.is_alphanumeric() || g.ch == '_'
        };
        if joined(k - 1) && joined(k + 1) && page.glyphs[k].sep == Sep::None {
            return Class::Word;
        }
    }
    Class::Other
}

/// Merges neighbouring quads (on one page) until at most `cap` remain,
/// always taking the pair that adds the least area.
fn shrink_to(quads: &mut BTreeMap<u32, Vec<Quad>>, cap: usize) {
    let area = |q: &[f64; 4]| (q[2] - q[0]) * (q[3] - q[1]);
    let merge = |a: &Quad, b: &Quad| {
        Quad([
            a.0[0].min(b.0[0]),
            a.0[1].min(b.0[1]),
            a.0[2].max(b.0[2]),
            a.0[3].max(b.0[3]),
        ])
    };
    while quads.values().map(Vec::len).sum::<usize>() > cap {
        let mut best: Option<(u32, usize, f64)> = None;
        for (&page, v) in quads.iter() {
            for i in 0..v.len().saturating_sub(1) {
                let m = merge(&v[i], &v[i + 1]);
                let cost = area(&m.0) - area(&v[i].0) - area(&v[i + 1].0);
                if best.is_none_or(|(_, _, c)| cost < c) {
                    best = Some((page, i, cost));
                }
            }
        }
        let Some((page, i, _)) = best else { break };
        let v = quads.get_mut(&page).expect("page was just found");
        let m = merge(&v[i], &v[i + 1]);
        v[i] = m;
        v.remove(i + 1);
    }
}
