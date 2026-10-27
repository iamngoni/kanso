//! Vim's motions: where each one takes the cursor, and whether an operator
//! over it takes the characters up to the target, up to and including it,
//! or whole lines. Word motions follow Vim's own stepping (`fwd_word`,
//! `bck_word` and `end_word` in its source), with the end of every line a
//! place of its own, so `dw` on a line's last word stops at its end.

use super::text::*;

/// How an operator over a motion takes the text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Up to the target, not including it.
    Exclusive,
    /// Up to and including the target.
    Inclusive,
    /// The whole lines from the cursor's to the target's.
    Linewise,
}

/// `f`, `F`, `t` or `T`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindKind {
    /// `f`: onto the character.
    To,
    /// `F`: back onto it.
    BackTo,
    /// `t`: up to just before it.
    Till,
    /// `T`: back to just after it.
    BackTill,
}

impl FindKind {
    fn reversed(self) -> FindKind {
        match self {
            FindKind::To => FindKind::BackTo,
            FindKind::BackTo => FindKind::To,
            FindKind::Till => FindKind::BackTill,
            FindKind::BackTill => FindKind::Till,
        }
    }

    fn forward(self) -> bool {
        matches!(self, FindKind::To | FindKind::Till)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    /// `w` and `W`.
    WordForward(bool),
    /// `b` and `B`.
    WordBack(bool),
    /// `e` and `E`.
    WordEnd(bool),
    /// `0`.
    LineStart,
    /// `^`.
    FirstNonBlank,
    /// `$`.
    LineEnd,
    /// `gg`: the first line, or line N.
    FirstLine,
    /// `G`: the last line, or line N.
    LastLine,
    /// `+` and Enter.
    NextLineStart,
    /// `-`.
    PrevLineStart,
    Find(FindKind, char),
    /// `;` (false) and `,` (true).
    RepeatFind(bool),
    /// `}`.
    ParagraphForward,
    /// `{`.
    ParagraphBack,
    /// `%`.
    Bracket,
    /// `ge` and `gE`.
    WordEndBack(bool),
    /// `H`, `M` and `L`: the first, middle and last lines in view.
    ViewTop,
    ViewMiddle,
    ViewBottom,
}

/// Where the cursor wants to be on lines it moves up and down through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Want {
    Column(usize),
    End,
}

/// What a motion needs besides the text: how many times, whether an
/// operator waits for it (which lets `l` and `w` reach a line's end), the
/// column `j` and `k` keep, and the last `f`, `t`, `F` or `T`.
pub struct Context {
    pub count: Option<usize>,
    pub operator: bool,
    pub want: Option<Want>,
    pub last_find: Option<(FindKind, char)>,
    /// The first and last lines (0-based) in view; the whole text when
    /// unknown.
    pub view: Option<(usize, usize)>,
}

/// Where `motion` from `from` goes, and how an operator takes it; `None`
/// when it can't go anywhere (an operator over it then does nothing).
pub fn eval(text: &str, from: usize, motion: Motion, cx: &Context) -> Option<(usize, Kind)> {
    let n = cx.count.unwrap_or(1).max(1);
    let from = floor_grapheme(text, from);
    match motion {
        Motion::Left => {
            let mut at = from;
            for _ in 0..n {
                match prev_in_line(text, at) {
                    Some(prev) => at = prev,
                    None => break,
                }
            }
            (at != from).then_some((at, Kind::Exclusive))
        }
        Motion::Right => {
            let mut at = from;
            for _ in 0..n {
                match next_in_line(text, at) {
                    Some(next) if cx.operator || !at_line_end(text, next) => at = next,
                    _ => break,
                }
            }
            (at != from).then_some((at, Kind::Exclusive))
        }
        Motion::Down | Motion::Up => {
            let mut start = line_start(text, from);
            for _ in 0..n {
                let next = if motion == Motion::Down {
                    next_line(text, start)
                } else {
                    prev_line(text, start)
                };
                match next {
                    Some(next) => start = next,
                    None => break,
                }
            }
            if start == line_start(text, from) {
                return None;
            }
            let at = match cx.want {
                Some(Want::End) => last_in_line(text, start),
                Some(Want::Column(col)) => at_column(text, start, col, false),
                None => at_column(text, start, column(text, from), false),
            };
            Some((at, Kind::Linewise))
        }
        Motion::WordForward(big) => {
            let at = word_forward(text, from, big, n, cx.operator);
            (at != from).then_some((at, Kind::Exclusive))
        }
        Motion::WordBack(big) => {
            let at = word_back(text, from, big, n);
            (at != from).then_some((at, Kind::Exclusive))
        }
        Motion::WordEnd(big) => {
            let end = word_end(text, from, big, n, false);
            // With an operator, running out of words takes the rest of the
            // text, as in Vim (`de` on the last character deletes it).
            let end = if cx.operator {
                Some(end.unwrap_or(text.len()))
            } else {
                end
            };
            end.map(|at| (at, Kind::Inclusive))
        }
        Motion::LineStart => Some((line_start(text, from), Kind::Exclusive)),
        Motion::FirstNonBlank => Some((first_non_blank(text, from), Kind::Exclusive)),
        Motion::LineEnd => {
            let mut start = line_start(text, from);
            for _ in 1..n {
                start = next_line(text, start)?;
            }
            Some((last_in_line(text, start), Kind::Inclusive))
        }
        Motion::FirstLine | Motion::LastLine => {
            let line = match (cx.count, motion) {
                (Some(n), _) => n.max(1) - 1,
                (None, Motion::FirstLine) => 0,
                (None, _) => usize::MAX,
            };
            Some((first_non_blank(text, nth_line(text, line)), Kind::Linewise))
        }
        Motion::NextLineStart | Motion::PrevLineStart => {
            let mut start = line_start(text, from);
            for _ in 0..n {
                start = if motion == Motion::NextLineStart {
                    next_line(text, start)?
                } else {
                    prev_line(text, start)?
                };
            }
            Some((first_non_blank(text, start), Kind::Linewise))
        }
        Motion::Find(kind, ch) => find(text, from, kind, ch, n, false),
        Motion::RepeatFind(reverse) => {
            let (kind, ch) = cx.last_find?;
            let kind = if reverse { kind.reversed() } else { kind };
            find(text, from, kind, ch, n, true)
        }
        Motion::ParagraphForward => Some(paragraph(text, from, n, true)),
        Motion::ParagraphBack => Some(paragraph(text, from, n, false)),
        Motion::Bracket => bracket(text, from).map(|at| (at, Kind::Inclusive)),
        Motion::WordEndBack(big) => {
            let at = word_end_back(text, from, big, n);
            (at != from).then_some((at, Kind::Inclusive))
        }
        Motion::ViewTop | Motion::ViewMiddle | Motion::ViewBottom => {
            let last_line = text.matches('\n').count();
            let (top, bottom) = match cx.view {
                Some((top, bottom)) => {
                    let top = top.min(last_line);
                    (top, bottom.clamp(top, last_line))
                }
                None => (0, last_line),
            };
            let line = match motion {
                Motion::ViewTop => (top + n - 1).min(bottom),
                Motion::ViewMiddle => top + (bottom - top) / 2,
                _ => bottom.saturating_sub(n - 1).max(top),
            };
            Some((first_non_blank(text, nth_line(text, line)), Kind::Linewise))
        }
    }
}

/// `ge`, `count` times: back to the end of the word before (Vim's
/// `bckend_word`); where it got to when the text runs out.
pub fn word_end_back(text: &str, from: usize, big: bool, count: usize) -> usize {
    let mut at = from;
    for _ in 0..count {
        let class = class_at(text, at, big);
        let Some((prev, _)) = dec(text, at) else {
            return at;
        };
        at = prev;
        // Back over the rest of the word the cursor was in.
        if class != 0 {
            while class_at(text, at, big) == class {
                match dec(text, at) {
                    Some((prev, _)) => at = prev,
                    None => return at,
                }
            }
        }
        // Back over blanks to the end of the word before; an empty line
        // is a word of its own.
        while class_at(text, at, big) == 0 {
            if is_empty_line(text, at) {
                break;
            }
            match dec(text, at) {
                Some((prev, _)) => at = prev,
                None => return at,
            }
        }
    }
    at
}

/// `w`, `count` times. With an operator waiting, the last word stops at its
/// line's end rather than going on to the next line's first word.
pub fn word_forward(text: &str, from: usize, big: bool, count: usize, operator: bool) -> usize {
    let mut at = from;
    for left in (0..count).rev() {
        let stop = operator && left == 0;
        let class = class_at(text, at, big);
        let Some((next, step)) = inc(text, at) else {
            return at;
        };
        at = next;
        if step >= 1 && stop {
            return at;
        }
        if class != 0 {
            while class_at(text, at, big) == class && !at_line_end(text, at) {
                let Some((next, step)) = inc(text, at) else {
                    return at;
                };
                at = next;
                if step >= 1 && stop {
                    return at;
                }
            }
        }
        while class_at(text, at, big) == 0 {
            if is_empty_line(text, at) {
                break;
            }
            let Some((next, step)) = inc(text, at) else {
                return at;
            };
            at = next;
            if step >= 1 && stop {
                return at;
            }
        }
    }
    at
}

/// `b`, `count` times.
pub fn word_back(text: &str, from: usize, big: bool, count: usize) -> usize {
    let mut at = from;
    for _ in 0..count {
        let Some((prev, _)) = dec(text, at) else {
            return at;
        };
        at = prev;
        let mut finished = false;
        while class_at(text, at, big) == 0 {
            if is_empty_line(text, at) {
                finished = true;
                break;
            }
            match dec(text, at) {
                Some((prev, _)) => at = prev,
                None => {
                    finished = true;
                    break;
                }
            }
        }
        if finished {
            continue;
        }
        let class = class_at(text, at, big);
        loop {
            match dec(text, at) {
                Some((prev, _))
                    if class_at(text, prev, big) == class && !at_line_end(text, prev) =>
                {
                    at = prev
                }
                _ => break,
            }
        }
    }
    at
}

/// `e`, `count` times; `None` when there is no word end ahead. With `stop`
/// (for `cw`), the end of the word under the cursor is as far as it goes,
/// even when the cursor is on it already.
pub fn word_end(text: &str, from: usize, big: bool, count: usize, stop: bool) -> Option<usize> {
    let mut at = from;
    for round in 0..count {
        // Only the first word may stop at the cursor (Vim clears `stop`).
        let stop = stop && round == 0;
        let class = class_at(text, at, big);
        let (next, _) = inc(text, at)?;
        at = next;
        if class != 0 && class_at(text, at, big) == class {
            at = skip_run(text, at, class, big);
        } else if !stop || class == 0 {
            while class_at(text, at, big) == 0 {
                at = inc(text, at)?.0;
            }
            let class = class_at(text, at, big);
            at = skip_run(text, at, class, big);
        }
        at = match dec(text, at) {
            Some((prev, _)) => prev,
            None => at,
        };
    }
    Some(at)
}

/// The first place after the run of `class` that `at` is in.
fn skip_run(text: &str, mut at: usize, class: u8, big: bool) -> usize {
    while class_at(text, at, big) == class && !at_line_end(text, at) {
        match inc(text, at) {
            Some((next, _)) => at = next,
            None => break,
        }
    }
    at
}

/// `f`, `F`, `t` or `T` for `ch`, the `count`th time on the line. A
/// repeated `t` doesn't stop where it already is, just before `ch`.
fn find(
    text: &str,
    from: usize,
    kind: FindKind,
    ch: char,
    count: usize,
    repeat: bool,
) -> Option<(usize, Kind)> {
    let start = line_start(text, from);
    let end = line_end(text, from);
    let matches = |at: usize| text[at..].starts_with(ch);
    let mut at = from;
    let mut found = None;
    let mut left = count;
    if kind.forward() {
        if repeat && kind == FindKind::Till {
            // `;` after `t` steps past the target it is already before.
            if let Some(next) = next_in_line(text, at)
                && next < end
                && matches(next)
            {
                at = next;
            }
        }
        while let Some(next) = next_in_line(text, at) {
            if next >= end {
                break;
            }
            at = next;
            if matches(at) {
                left -= 1;
                if left == 0 {
                    found = Some(at);
                    break;
                }
            }
        }
        let target = found?;
        Some(match kind {
            FindKind::Till => (prev_in_line(text, target).unwrap_or(start), Kind::Inclusive),
            _ => (target, Kind::Inclusive),
        })
    } else {
        if repeat
            && kind == FindKind::BackTill
            && let Some(prev) = prev_in_line(text, at)
            && matches(prev)
        {
            at = prev;
        }
        while let Some(prev) = prev_in_line(text, at) {
            at = prev;
            if matches(at) {
                left -= 1;
                if left == 0 {
                    found = Some(at);
                    break;
                }
            }
        }
        let target = found?;
        let at = match kind {
            FindKind::BackTill => next_in_line(text, target).unwrap_or(target),
            _ => target,
        };
        Some((at, Kind::Exclusive))
    }
}

/// `}` or `{`: the next (or previous) empty line after some text, `count`
/// times; the end of the last line, or the start of the first, when there
/// is none.
fn paragraph(text: &str, from: usize, count: usize, forward: bool) -> (usize, Kind) {
    let mut line = line_start(text, from);
    for _ in 0..count {
        let mut skipped = false;
        let mut first = true;
        loop {
            if !is_empty_line(text, line) {
                skipped = true;
            }
            if !first && skipped && is_empty_line(text, line) {
                break;
            }
            first = false;
            let next = if forward {
                next_line(text, line)
            } else {
                prev_line(text, line)
            };
            match next {
                Some(next) => line = next,
                None => {
                    if forward {
                        // The end of the last line, inclusive.
                        return (last_in_line(text, line), Kind::Inclusive);
                    }
                    return (line, Kind::Exclusive);
                }
            }
        }
    }
    (line, Kind::Exclusive)
}

/// `%`: the bracket matching the first one at or after the cursor on its
/// line.
pub fn bracket(text: &str, from: usize) -> Option<usize> {
    let end = line_end(text, from);
    let (at, ch) = text[from..end]
        .char_indices()
        .find(|(_, c)| "()[]{}".contains(*c))
        .map(|(i, c)| (from + i, c))?;
    let (open, close, forward) = match ch {
        '(' => ('(', ')', true),
        '[' => ('[', ']', true),
        '{' => ('{', '}', true),
        ')' => ('(', ')', false),
        ']' => ('[', ']', false),
        _ => ('{', '}', false),
    };
    matching(text, at, open, close, forward)
}

/// The bracket that matches the one at `at`, counting nesting.
pub fn matching(text: &str, at: usize, open: char, close: char, forward: bool) -> Option<usize> {
    let mut depth = 0usize;
    if forward {
        for (i, c) in text[at..].char_indices() {
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    return Some(at + i);
                }
            }
        }
    } else {
        let end = at + text[at..].chars().next()?.len_utf8();
        for (i, c) in text[..end].char_indices().rev() {
            if c == close {
                depth += 1;
            } else if c == open {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}
