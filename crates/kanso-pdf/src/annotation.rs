//! Reading and writing the annotation block: the `kanso-pdf` fence in a body.
//!
//! The writer is canonical: a block that was written by it parses and
//! serialises back to the same bytes. A block that cannot be read is never
//! rewritten; the callers keep the text as it is.

use std::collections::HashSet;
use std::ops::Range;

use serde_json::Value;

use crate::PdfError;
use crate::model::{
    BLOCK_FENCE, BLOCK_VERSION, Highlight, MAX_BLOCK_CHARS, MAX_COMMENT_CHARS, MAX_QUADS,
    MAX_QUOTE_CHARS, MAX_REMOVED, PdfBlock,
};
use crate::quote::Edit;

const ID_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
/// Pages a document may have (the worker refuses more).
const MAX_PAGE: u32 = 100_000;

/// A fresh highlight id: eight base32 characters.
pub fn new_highlight_id() -> String {
    (0..8)
        .map(|_| ID_ALPHABET[fastrand::usize(..ID_ALPHABET.len())] as char)
        .collect()
}

fn valid_id(id: &str) -> bool {
    (1..=16).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

/// Checks one highlight against the block's rules.
pub fn validate_highlight(h: &Highlight) -> Result<(), PdfError> {
    let bad = |m: &str| Err(PdfError::Invalid(m.to_string()));
    if !valid_id(&h.id) {
        return bad("A highlight id is not valid.");
    }
    if h.page == 0 || h.page > MAX_PAGE {
        return bad("A highlight names a page that does not exist.");
    }
    if h.quads.is_empty() || h.quads.values().any(Vec::is_empty) {
        return bad("A highlight has no area.");
    }
    if h.quads.keys().any(|p| *p == 0 || *p > MAX_PAGE) || h.quads.keys().next() != Some(&h.page) {
        return bad("A highlight's first page does not match its areas.");
    }
    if h.quads.values().map(Vec::len).sum::<usize>() > MAX_QUADS {
        return bad("A highlight covers too many lines.");
    }
    if h.quads.values().flatten().any(|q| !q.is_valid()) {
        return bad("A highlight has an area outside its page.");
    }
    if h.text.chars().count() > MAX_QUOTE_CHARS {
        return bad("The highlighted text is too long.");
    }
    if h.comment
        .as_ref()
        .is_some_and(|c| c.chars().count() > MAX_COMMENT_CHARS)
    {
        return bad("The comment is too long.");
    }
    Ok(())
}

impl PdfBlock {
    /// Parses and validates the JSON inside a `kanso-pdf` fence.
    pub fn parse(json: &str) -> Result<PdfBlock, PdfError> {
        if json.chars().count() > MAX_BLOCK_CHARS {
            return Err(PdfError::Malformed("the block is too large".into()));
        }
        let block: PdfBlock =
            serde_json::from_str(json).map_err(|e| PdfError::Malformed(e.to_string()))?;
        if block.version > BLOCK_VERSION {
            return Err(PdfError::NewerVersion(block.version));
        }
        if block.version == 0 || !block.file.starts_with("file:") || block.file.len() <= 5 {
            return Err(PdfError::Malformed("it names no attachment".into()));
        }
        if block.removed.len() > MAX_REMOVED || !block.removed.iter().all(|id| valid_id(id)) {
            return Err(PdfError::Malformed(
                "its deleted highlights are not valid".into(),
            ));
        }
        let mut seen = HashSet::new();
        for h in &block.highlights {
            validate_highlight(h)?;
            if !seen.insert(h.id.as_str()) {
                return Err(PdfError::Malformed("two highlights share an id".into()));
            }
        }
        Ok(block)
    }

    /// The canonical JSON for the fence, refusing a block over the cap.
    pub fn to_json(&self) -> Result<String, PdfError> {
        let out = write_block(self);
        if out.chars().count() > MAX_BLOCK_CHARS {
            return Err(PdfError::BlockFull);
        }
        Ok(out)
    }

    /// The whole fence, ready to put in a body.
    pub fn to_fence(&self) -> Result<String, PdfError> {
        Ok(format!("```{BLOCK_FENCE}\n{}\n```", self.to_json()?))
    }

    /// Adds a highlight, refusing one that is invalid or does not fit.
    pub fn add_highlight(&mut self, h: Highlight) -> Result<(), PdfError> {
        validate_highlight(&h)?;
        if self.highlights.iter().any(|x| x.id == h.id) {
            return Err(PdfError::Invalid("That highlight already exists.".into()));
        }
        self.highlights.push(h);
        if let Err(e) = self.to_json() {
            self.highlights.pop();
            return Err(e);
        }
        Ok(())
    }

    /// Removes a highlight; whether it was there. Its id is remembered in
    /// `removed`, so a merge with a version that still has it (another
    /// device's, another editor's) leaves it deleted.
    pub fn remove_highlight(&mut self, id: &str) -> bool {
        let n = self.highlights.len();
        self.highlights.retain(|h| h.id != id);
        let gone = self.highlights.len() != n;
        if gone {
            crate::merge::remember_removed(&mut self.removed, id);
        }
        gone
    }

    /// Sets or clears a highlight's comment (blank clears it).
    pub fn set_comment(&mut self, id: &str, comment: Option<&str>) -> Result<(), PdfError> {
        let comment = comment.map(str::trim).filter(|c| !c.is_empty());
        if comment.is_some_and(|c| c.chars().count() > MAX_COMMENT_CHARS) {
            return Err(PdfError::Invalid("The comment is too long.".into()));
        }
        let h = self
            .highlights
            .iter_mut()
            .find(|h| h.id == id)
            .ok_or_else(|| PdfError::Invalid("That highlight no longer exists.".into()))?;
        let old = h.comment.take();
        h.comment = comment.map(str::to_string);
        h.rev = h.rev.saturating_add(1);
        if let Err(e) = self.to_json() {
            if let Some(h) = self.highlights.iter_mut().find(|h| h.id == id) {
                h.comment = old;
                h.rev = h.rev.saturating_sub(1);
            }
            return Err(e);
        }
        Ok(())
    }
}

fn json(v: &impl serde::Serialize) -> String {
    serde_json::to_string(v).unwrap_or_else(|_| "null".into())
}

fn write_block(b: &PdfBlock) -> String {
    let mut o = String::from("{\n");
    o.push_str(&format!("  \"file\": {},\n", json(&b.file)));
    o.push_str(&format!("  \"md5\": {},\n", json(&b.md5)));
    o.push_str(&format!("  \"version\": {},\n", b.version));
    if b.highlights.is_empty() {
        o.push_str("  \"highlights\": [],\n");
    } else {
        o.push_str("  \"highlights\": [\n");
        let items: Vec<String> = b.highlights.iter().map(write_highlight).collect();
        o.push_str(&items.join(",\n"));
        o.push_str("\n  ],\n");
    }
    let mut tail: Vec<String> = Vec::new();
    if b.ink.is_empty() {
        tail.push("  \"ink\": []".into());
    } else {
        let items: Vec<String> = b.ink.iter().map(|v| format!("    {}", json(v))).collect();
        tail.push(format!("  \"ink\": [\n{}\n  ]", items.join(",\n")));
    }
    if let Some(f) = &b.ink_file {
        tail.push(format!("  \"inkFile\": {}", json(f)));
    }
    if !b.removed.is_empty() {
        tail.push(format!("  \"removed\": {}", json(&b.removed)));
    }
    for (k, v) in &b.extra {
        tail.push(format!("  {}: {}", json(k), json(v)));
    }
    o.push_str(&tail.join(",\n"));
    o.push_str("\n}");
    o
}

fn write_highlight(h: &Highlight) -> String {
    let quads: Vec<String> = h
        .quads
        .iter()
        .map(|(page, qs)| {
            let qs: Vec<String> = qs
                .iter()
                .map(|q| {
                    let v: Vec<String> = q.0.iter().map(json).collect();
                    format!("[{}]", v.join(", "))
                })
                .collect();
            format!("\"{page}\": [{}]", qs.join(", "))
        })
        .collect();
    let mut f: Vec<String> = vec![
        format!("      \"id\": {}", json(&h.id)),
        format!("      \"page\": {}", h.page),
        format!("      \"quads\": {{{}}}", quads.join(", ")),
        format!("      \"text\": {}", json(&h.text)),
        format!("      \"color\": {}", json(&h.color)),
    ];
    if let Some(c) = &h.comment {
        f.push(format!("      \"comment\": {}", json(c)));
    }
    f.push(format!("      \"createdAt\": {}", h.created_at));
    if h.rev > 0 {
        f.push(format!("      \"rev\": {}", h.rev));
    }
    for (k, v) in &h.extra {
        f.push(format!("      {}: {}", json(k), json(v)));
    }
    format!("    {{\n{}\n    }}", f.join(",\n"))
}

/// Where a `kanso-pdf` fence sits in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSpan {
    /// The whole fence, from its opening line to the end of its closing line
    /// (without the final newline).
    pub range: Range<usize>,
    /// The text between the fences.
    pub json: Range<usize>,
}

fn fence_open(line: &str) -> Option<(char, usize, &str)> {
    let t = line.trim_start_matches(' ');
    if line.len() - t.len() > 3 {
        return None;
    }
    let c = t.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let n = t.chars().take_while(|x| *x == c).count();
    if n < 3 {
        return None;
    }
    let info = t[n..].trim();
    if c == '`' && info.contains('`') {
        return None;
    }
    Some((c, n, info))
}

fn fence_close(line: &str, c: char, n: usize) -> bool {
    let t = line.trim_start_matches(' ');
    line.len() - t.len() <= 3 && {
        let k = t.chars().take_while(|x| *x == c).count();
        k >= n && t[k..].trim().is_empty()
    }
}

/// Every top-level `kanso-pdf` fence in a body, in order. A fence nested in
/// another code block is text, not a block.
pub fn find_blocks(body: &str) -> Vec<BlockSpan> {
    let mut out = Vec::new();
    // (char, len, is_ours, fence start, content start)
    let mut open: Option<(char, usize, bool, usize, usize)> = None;
    let mut pos = 0;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        let end = pos + raw.len();
        match open {
            None => {
                if let Some((c, n, info)) = fence_open(line) {
                    open = Some((c, n, info == BLOCK_FENCE, pos, end));
                }
            }
            Some((c, n, ours, start, content)) => {
                if fence_close(line, c, n) {
                    if ours {
                        out.push(BlockSpan {
                            range: start..pos + line.len(),
                            json: content..pos.max(content),
                        });
                    }
                    open = None;
                }
            }
        }
        pos = end;
    }
    // An unclosed fence runs to the end of the body (CommonMark). It is a
    // block that can't be read, never a place to append after: text added
    // below it would be inside it.
    if let Some((_, _, true, start, content)) = open {
        out.push(BlockSpan {
            range: start..body.len(),
            json: content.min(body.len())..body.len(),
        });
    }
    out
}

/// The edit that saves `block` into `body`: it replaces the block for the same
/// PDF, or appends one. An unreadable block for the same PDF is an error, so
/// the caller never overwrites what it could not read.
pub fn upsert_block(body: &str, block: &PdfBlock) -> Result<Edit, PdfError> {
    let fence = block.to_fence()?;
    for span in find_blocks(body) {
        let text = body[span.json.clone()].trim();
        match PdfBlock::parse(text) {
            Ok(old) if old.file == block.file => {
                return Ok(Edit {
                    range: span.range,
                    insert: fence,
                });
            }
            Ok(_) => {}
            Err(e) => {
                // Broken JSON (a hand edit, a lost brace) can't say which PDF
                // it is for; if it mentions this one, keep away from it, or
                // a second block would be added and the first left to rot.
                let names_us = match serde_json::from_str::<Value>(text) {
                    Ok(v) => v.get("file").and_then(Value::as_str) == Some(block.file.as_str()),
                    Err(_) => text.contains(&json(&block.file)),
                };
                if names_us {
                    return Err(e);
                }
            }
        }
    }
    let end = body.trim_end().len();
    let insert = if end == 0 {
        fence
    } else {
        format!("\n\n{fence}")
    };
    Ok(Edit {
        range: end..end,
        insert,
    })
}

/// The block for a PDF, if the body has one that reads cleanly.
pub fn block_for_file(body: &str, file: &str) -> Option<PdfBlock> {
    find_blocks(body)
        .into_iter()
        .filter_map(|s| PdfBlock::parse(body[s.json].trim()).ok())
        .find(|b| b.file == file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Color, Quad};
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn highlight(id: &str, page: u32) -> Highlight {
        let mut quads = BTreeMap::new();
        quads.insert(page, vec![Quad::new(0.1, 0.1, 0.5, 0.2)]);
        Highlight {
            id: id.into(),
            page,
            quads,
            text: "Revenue rose".into(),
            color: Color::Yellow,
            comment: None,
            created_at: 1_761_212_345_678,
            rev: 0,
            extra: Default::default(),
        }
    }

    const SPEC: &str = r#"{
  "file": "file:abc123",
  "md5": "0f1e",
  "version": 1,
  "highlights": [
    {
      "id": "h7k2m9",
      "page": 4,
      "quads": {"4": [[0.118, 0.071, 0.526, 0.087], [0.118, 0.096, 0.327, 0.112]]},
      "text": "Revenue rose 12 percent on the quarter, led by subscriptions.",
      "color": "yellow",
      "comment": "Check against the board deck.",
      "createdAt": 1761212345678
    }
  ],
  "ink": []
}"#;

    #[test]
    fn the_spec_example_is_byte_identical() {
        let b = PdfBlock::parse(SPEC).unwrap();
        assert_eq!(b.to_json().unwrap(), SPEC);
        assert_eq!(b.summary().unwrap(), "1 highlight · 1 comment");
    }

    #[test]
    fn a_cross_page_highlight_round_trips() {
        let mut b = PdfBlock::new("file:x", "m");
        let mut h = highlight("aaaa2222", 4);
        h.quads.insert(5, vec![Quad::new(0.0, 0.0, 0.4, 0.05)]);
        b.add_highlight(h.clone()).unwrap();
        let back = PdfBlock::parse(&b.to_json().unwrap()).unwrap();
        assert_eq!(back.highlights[0], h);
    }

    #[test]
    fn unknown_fields_survive_a_save() {
        let s = SPEC.replace("\"ink\": []", "\"ink\": [],\n  \"zoom\": 2");
        let b = PdfBlock::parse(&s).unwrap();
        assert_eq!(b.to_json().unwrap(), s);
    }

    #[test]
    fn malformed_blocks_are_rejected() {
        for bad in [
            "{",
            "{}",
            &SPEC.replace("\"version\": 1", "\"version\": 2"),
            &SPEC.replace("\"page\": 4", "\"page\": 0"),
            &SPEC.replace("\"page\": 4", "\"page\": 3"),
            &SPEC.replace("0.526", "1.5"),
            &SPEC.replace("file:abc123", "abc123"),
            &SPEC.replace("\"h7k2m9\"", "\"BAD ID\""),
        ] {
            assert!(PdfBlock::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn caps_are_enforced() {
        let mut b = PdfBlock::new("file:x", "");
        let mut h = highlight("a1", 1);
        h.quads
            .insert(1, vec![Quad::new(0.0, 0.0, 0.1, 0.1); MAX_QUADS + 1]);
        assert!(b.add_highlight(h).is_err());
        let mut h = highlight("a2", 1);
        h.text = "x".repeat(MAX_QUOTE_CHARS + 1);
        assert!(b.add_highlight(h).is_err());
        b.add_highlight(highlight("a3", 1)).unwrap();
        assert!(
            b.set_comment("a3", Some(&"c".repeat(MAX_COMMENT_CHARS + 1)))
                .is_err()
        );
        assert!(b.highlights[0].comment.is_none());
    }

    #[test]
    fn a_full_block_refuses_the_next_highlight() {
        let mut b = PdfBlock::new("file:x", "");
        let mut n = 0;
        loop {
            let mut h = highlight(&format!("h{n:06}"), 1);
            h.comment = Some("c".repeat(MAX_COMMENT_CHARS));
            n += 1;
            match b.add_highlight(h) {
                Ok(()) => {}
                Err(PdfError::BlockFull) => break,
                Err(e) => panic!("{e}"),
            }
        }
        assert!(b.to_json().is_ok());
        assert!(b.highlights.len() < n);
    }

    #[test]
    fn fences_are_found_outside_other_code() {
        let body = format!(
            "intro\n\n````md\n```kanso-pdf\n{{}}\n```\n````\n\n```kanso-pdf\n{SPEC}\n```\n\ntail"
        );
        let spans = find_blocks(&body);
        assert_eq!(spans.len(), 1);
        assert_eq!(body[spans[0].json.clone()].trim(), SPEC);
        assert!(body[spans[0].range.clone()].ends_with("```"));
    }

    #[test]
    fn upsert_appends_then_replaces() {
        let mut b = PdfBlock::new("file:abc", "m");
        let body = "# Notes\n\ntext\n";
        let e = upsert_block(body, &b).unwrap();
        let body = e.apply(body);
        assert_eq!(block_for_file(&body, "file:abc").unwrap(), b);
        b.add_highlight(highlight("zz", 2)).unwrap();
        let e = upsert_block(&body, &b).unwrap();
        let body2 = e.apply(&body);
        assert_eq!(find_blocks(&body2).len(), 1);
        assert_eq!(block_for_file(&body2, "file:abc").unwrap(), b);
        assert!(body2.starts_with("# Notes\n\ntext\n\n```kanso-pdf"));
    }

    #[test]
    fn an_unreadable_block_for_the_same_pdf_is_not_overwritten() {
        let body = "```kanso-pdf\n{\"file\": \"file:abc\", \"version\": 9}\n```";
        let b = PdfBlock::new("file:abc", "");
        assert!(upsert_block(body, &b).is_err());
        let other = PdfBlock::new("file:other", "");
        assert!(upsert_block(body, &other).is_ok());
    }

    #[test]
    fn a_broken_or_unclosed_block_for_the_same_pdf_is_not_appended_to() {
        // A lost brace: not JSON, but it names the file.
        let body = "```kanso-pdf\n{\"file\": \"file:abc\", \"version\": 1,\n```";
        assert!(upsert_block(body, &PdfBlock::new("file:abc", "")).is_err());
        assert!(upsert_block(body, &PdfBlock::new("file:other", "")).is_ok());
        // A lost closing fence: what follows would be inside the block.
        let open = format!("text\n\n```kanso-pdf\n{SPEC}\n");
        assert_eq!(find_blocks(&open).len(), 1);
        assert!(upsert_block(&open, &PdfBlock::new("file:abc123", "")).is_ok());
        let e = upsert_block(&open, &PdfBlock::new("file:abc123", "")).unwrap();
        // The readable unclosed block is replaced, closing it.
        let fixed = e.apply(&open);
        assert_eq!(find_blocks(&fixed).len(), 1);
        assert!(fixed.ends_with("```"));
        let broken = "```kanso-pdf\n{\"file\": \"file:abc\"\n";
        assert!(upsert_block(broken, &PdfBlock::new("file:abc", "")).is_err());
    }

    #[test]
    fn ids_are_eight_base32() {
        let id = new_highlight_id();
        assert_eq!(id.len(), 8);
        assert!(valid_id(&id));
    }

    fn arb_highlight() -> impl Strategy<Value = Highlight> {
        (
            "[a-z0-9]{1,16}",
            1u32..50,
            0u32..3,
            prop::collection::vec(
                (0u32..=10_000, 0u32..=10_000, 0u32..=10_000, 0u32..=10_000),
                1..4,
            ),
            any::<String>().prop_map(|s| s.chars().take(100).collect::<String>()),
            prop::sample::select(Color::ALL.to_vec()),
            prop::option::of(any::<String>().prop_map(|s| s.chars().take(100).collect::<String>())),
            0i64..i64::MAX / 2,
        )
            .prop_map(|(id, page, span, qs, text, color, comment, created_at)| {
                let mut quads = BTreeMap::new();
                for p in page..=page + span {
                    quads.insert(
                        p,
                        qs.iter()
                            .map(|(a, b, c, d)| {
                                let f = |v: &u32| f64::from(*v) / 10_000.0;
                                Quad::new(f(a), f(b), f(c), f(d))
                            })
                            .collect(),
                    );
                }
                Highlight {
                    id,
                    page,
                    quads,
                    text,
                    color,
                    comment,
                    created_at,
                    rev: 0,
                    extra: Default::default(),
                }
            })
    }

    proptest! {
        #[test]
        fn blocks_round_trip_byte_identically(
            hs in prop::collection::vec(arb_highlight(), 0..6),
            md5 in "[0-9a-f]{0,32}",
        ) {
            let mut b = PdfBlock::new("file:abc123", md5);
            let mut ids = HashSet::new();
            for h in hs {
                if ids.insert(h.id.clone()) {
                    b.add_highlight(h).unwrap();
                }
            }
            let json = b.to_json().unwrap();
            let back = PdfBlock::parse(&json).unwrap();
            prop_assert_eq!(&back, &b);
            prop_assert_eq!(back.to_json().unwrap(), json);
        }

        #[test]
        fn parse_never_panics_and_valid_blocks_resave_stably(s in "\\PC{0,300}") {
            if let Ok(b) = PdfBlock::parse(&s) {
                let j = b.to_json().unwrap();
                prop_assert_eq!(PdfBlock::parse(&j).unwrap().to_json().unwrap(), j);
            }
        }

        #[test]
        fn find_blocks_never_panics(s in "[`~a-z\\n \\-]{0,200}") {
            for span in find_blocks(&s) {
                prop_assert!(span.range.end <= s.len());
                prop_assert!(span.json.start <= span.json.end);
            }
        }
    }
}
