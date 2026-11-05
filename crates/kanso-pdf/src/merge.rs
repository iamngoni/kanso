//! Two versions of a PDF's highlights become one (issue #43, step 6).
//!
//! A note's body is last-writer-wins between devices, and a shared note's
//! text is merged character by character. Neither may lose a highlight: two
//! devices highlighting the same PDF at once, or a save from the reader while
//! another editor of the note added one, must end with both. So every place a
//! block is written over another version of itself unites the two first:
//!
//! * highlights are united by id; of two versions of one, the one changed
//!   more often (`rev`) wins, and on a tie the larger as written, so both
//!   devices pick the same;
//! * a deleted highlight's id stays in `removed`, and an id there is never
//!   brought back by the other version;
//! * a block that can't be read is never written over, and several blocks
//!   for the same PDF (two editors each wrote theirs) are joined into the
//!   first.
//!
//! [`merge_blocks`] is commutative, associative and idempotent, so devices
//! that meet in any order end with the same highlights.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::PdfError;
use crate::annotation::{BlockSpan, find_blocks};
use crate::model::{Highlight, MAX_REMOVED, PdfBlock};
use crate::quote::Edit;

/// Adds `id` to a sorted list of deleted ids, keeping it within
/// [`MAX_REMOVED`] (the smallest ids go first, the same on every device).
pub fn remember_removed(removed: &mut Vec<String>, id: &str) {
    if let Err(at) = removed.binary_search_by(|x| x.as_str().cmp(id)) {
        removed.insert(at, id.to_owned());
    }
    if removed.len() > MAX_REMOVED {
        let over = removed.len() - MAX_REMOVED;
        removed.drain(..over);
    }
}

fn written(v: &impl serde::Serialize) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

/// The larger of two values as written: an order both devices agree on.
fn larger<'a>(a: &'a Value, b: &'a Value) -> &'a Value {
    if written(a) >= written(b) { a } else { b }
}

pub(crate) fn merge_maps(a: &Map<String, Value>, b: &Map<String, Value>) -> Map<String, Value> {
    merge_extra(a, b)
}

fn merge_extra(a: &Map<String, Value>, b: &Map<String, Value>) -> Map<String, Value> {
    let mut out = a.clone();
    for (k, v) in b {
        let pick = match a.get(k) {
            Some(mine) => larger(mine, v).clone(),
            None => v.clone(),
        };
        out.insert(k.clone(), pick);
    }
    out
}

/// Which of two versions of one highlight wins.
fn winner<'a>(a: &'a Highlight, b: &'a Highlight) -> &'a Highlight {
    match a.rev.cmp(&b.rev) {
        std::cmp::Ordering::Greater => a,
        std::cmp::Ordering::Less => b,
        std::cmp::Ordering::Equal => {
            if written(a) >= written(b) {
                a
            } else {
                b
            }
        }
    }
}

/// The union of two versions of the same PDF's block (`a.file` is kept).
/// Pure; commutative, associative and idempotent.
pub fn merge_blocks(a: &PdfBlock, b: &PdfBlock) -> PdfBlock {
    let mut removed: BTreeSet<String> = a.removed.iter().chain(&b.removed).cloned().collect();
    while removed.len() > MAX_REMOVED {
        removed.pop_first();
    }
    let mut by_id: BTreeMap<&str, &Highlight> = BTreeMap::new();
    for h in a.highlights.iter().chain(&b.highlights) {
        if removed.contains(&h.id) {
            continue;
        }
        let pick = match by_id.get(h.id.as_str()) {
            Some(old) => winner(old, h),
            None => h,
        };
        by_id.insert(&h.id, pick);
    }
    let mut highlights: Vec<Highlight> = by_id.into_values().cloned().collect();
    // In the order they were made, so the newest is drawn on top.
    highlights.sort_by(|p, q| (p.created_at, &p.id).cmp(&(q.created_at, &q.id)));
    PdfBlock {
        file: a.file.clone(),
        md5: if a.md5.is_empty() || (!b.md5.is_empty() && b.md5 > a.md5) {
            b.md5.clone()
        } else {
            a.md5.clone()
        },
        version: a.version.max(b.version),
        highlights,
        ink: crate::ink::merge_pages(&a.ink, &b.ink),
        ink_file: a.ink_file.clone().max(b.ink_file.clone()),
        removed: removed.into_iter().collect(),
        extra: merge_extra(&a.extra, &b.extra),
    }
}

/// What a fence in a body is, for one PDF.
pub(crate) enum Fence {
    /// This PDF's block, read.
    Ours(PdfBlock),
    /// This PDF's block, but it can't be read.
    Broken(PdfError),
    /// Another PDF's, or not one at all.
    Other,
}

/// What the fence `span` of `body` is for `file`. A block that can't be read
/// is this PDF's when it names it, as JSON or (broken JSON) as text.
pub(crate) fn fence_for(body: &str, span: &BlockSpan, file: &str) -> Fence {
    let text = body[span.json.clone()].trim();
    match PdfBlock::parse(text) {
        Ok(block) if block.file == file => Fence::Ours(block),
        Ok(_) => Fence::Other,
        Err(e) => {
            let names_us = match serde_json::from_str::<Value>(text) {
                Ok(v) => v.get("file").and_then(Value::as_str) == Some(file),
                Err(_) => text.contains(&written(&file)),
            };
            if names_us {
                Fence::Broken(e)
            } else {
                Fence::Other
            }
        }
    }
}

/// This PDF's fences in `body`, read; an error when one can't be.
fn ours(body: &str, file: &str) -> Result<Vec<(BlockSpan, PdfBlock)>, PdfError> {
    let mut out = Vec::new();
    for span in find_blocks(body) {
        match fence_for(body, &span, file) {
            Fence::Ours(block) => out.push((span, block)),
            Fence::Broken(e) => return Err(e),
            Fence::Other => {}
        }
    }
    Ok(out)
}

/// One replacement of a span of the body.
struct Swap {
    range: std::ops::Range<usize>,
    insert: String,
}

/// The replacements that put `block` into this PDF's first fence and take
/// out the others (`spans` in order, at least one).
fn swaps(body: &str, spans: &[BlockSpan], block: &PdfBlock) -> Result<Vec<Swap>, PdfError> {
    let fence = block.to_fence()?;
    let mut out = vec![Swap {
        range: spans[0].range.clone(),
        insert: fence,
    }];
    for span in &spans[1..] {
        // The blank line that set it apart goes with it.
        let start = if body[..span.range.start].ends_with("\n\n") {
            span.range.start - 2
        } else {
            span.range.start
        };
        out.push(Swap {
            range: start..span.range.end,
            insert: String::new(),
        });
    }
    Ok(out)
}

fn apply(body: &str, mut swaps: Vec<Swap>) -> String {
    swaps.sort_by_key(|s| std::cmp::Reverse(s.range.start));
    let mut out = body.to_owned();
    for s in swaps {
        out.replace_range(s.range, &s.insert);
    }
    out
}

/// The one edit that turns `old` into `new`: what lies between their common
/// start and their common end.
pub fn edit_between(old: &str, new: &str) -> Edit {
    let mut p = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(p) || !new.is_char_boundary(p) {
        p -= 1;
    }
    let room = old.len().min(new.len()) - p;
    let mut s = old
        .bytes()
        .rev()
        .zip(new.bytes().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(old.len() - s) || !new.is_char_boundary(new.len() - s) {
        s -= 1;
    }
    Edit {
        range: p..old.len() - s,
        insert: new[p..new.len() - s].to_owned(),
    }
}

/// The edit that saves `block` into `body`, united with whatever version of
/// it the body holds now (another editor's highlights are kept, and several
/// blocks for this PDF become one). `None` when the body already says it.
/// A block for this PDF that can't be read is an error: it is never written
/// over.
pub fn save_into(body: &str, block: &PdfBlock) -> Result<Option<Edit>, PdfError> {
    let found = ours(body, &block.file)?;
    if found.is_empty() {
        let fence = block.to_fence()?;
        let end = body.trim_end().len();
        let insert = if end == 0 {
            fence
        } else {
            format!("\n\n{fence}")
        };
        return Ok(Some(Edit {
            range: end..end,
            insert,
        }));
    }
    let merged = found
        .iter()
        .fold(block.clone(), |acc, (_, b)| merge_blocks(&acc, b));
    if found.len() == 1 && found[0].1 == merged {
        return Ok(None);
    }
    let spans: Vec<BlockSpan> = found.into_iter().map(|(s, _)| s).collect();
    let next = apply(body, swaps(body, &spans, &merged)?);
    Ok((next != body).then(|| edit_between(body, &next)))
}

/// All of this PDF's readable blocks in `body` united; `None` when it has
/// none. An error when one of them can't be read.
pub(crate) fn united(body: &str, file: &str) -> Result<Option<PdfBlock>, PdfError> {
    let found = ours(body, file)?;
    let mut blocks = found.into_iter().map(|(_, b)| b);
    let Some(first) = blocks.next() else {
        return Ok(None);
    };
    Ok(Some(blocks.fold(first, |acc, b| merge_blocks(&acc, &b))))
}

/// The body `winner` with the highlights of `loser` folded into each PDF's
/// block they both have: what sync keeps when one device's version of a note
/// wins over another's. Text outside the blocks is `winner`'s; a PDF's block
/// that can't be read on either side is left as it is, and a PDF only one
/// side has a block for is left alone (the other side may have taken it out
/// on purpose; its version is in the note's history). `None` when nothing
/// changes, or a block would outgrow its cap.
pub fn merge_bodies(winner: &str, loser: &str) -> Option<String> {
    let mut files: Vec<String> = Vec::new();
    for span in find_blocks(winner) {
        let text = winner[span.json.clone()].trim();
        if let Ok(block) = PdfBlock::parse(text)
            && !files.contains(&block.file)
        {
            files.push(block.file);
        }
    }
    let mut all = Vec::new();
    for file in files {
        let (Ok(mine), Ok(Some(theirs))) = (ours(winner, &file), united(loser, &file)) else {
            continue;
        };
        let merged = mine
            .iter()
            .fold(theirs, |acc, (_, b)| merge_blocks(&acc, b));
        if mine.len() == 1 && mine[0].1 == merged {
            continue;
        }
        let spans: Vec<BlockSpan> = mine.into_iter().map(|(s, _)| s).collect();
        all.extend(swaps(winner, &spans, &merged).ok()?);
    }
    if all.is_empty() {
        return None;
    }
    let out = apply(winner, all);
    (out != winner).then_some(out)
}

#[cfg(test)]
mod tests;
