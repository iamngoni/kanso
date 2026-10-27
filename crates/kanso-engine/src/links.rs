//! Links between notes, without a window: the titles wiki-links resolve
//! against, how the preview writes them, `[[` autocomplete, the link a note
//! copies for itself, and the offer to update links after a rename.
//!
//! The preview is handed the note's Markdown with each wiki-link written as
//! an ordinary link ([`rewrite_for_preview`]): to an ID link
//! ([`NOTE_LINK_PREFIX`]`<id>`) when its title names a note, so it opens
//! like any note link, and to a `kanso-new:` link when it names none.
//!
//! Autocomplete ([`wiki_choices`]) writes `[Title](<ID link>)`, which names
//! the note by ID and so survives renames. `#tag` autocomplete is in
//! [`tags`].
//!
//! Everything here is a pure function of the text and of the titles the
//! caller passes in; the caller decides which notes those are, and leaves
//! out locked and trashed ones (the store's `note_titles` does).

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

use kanso_core::links::{
    LINK_SCHEME, NOTE_LINK_PREFIX, code_ranges, normalize_title, parse_target, wiki_links,
};
use kanso_store::Store;

pub mod mention;
pub mod tags;

pub use kanso_core::links::retitle_wiki_links;

/// The scheme of the links unresolved wiki-links become in the preview.
pub const NEW_NOTE_SCHEME: &str = "kanso-new:";
/// The most titles autocomplete offers at once.
pub const COMPLETION_LIMIT: usize = 12;
/// How far back from the caret a `[[` is looked for.
const LOOK_BACK: usize = 400;

/// The titles wiki-links resolve against: every live note outside the
/// trash, the first created first.
#[derive(Clone, Debug, Default)]
pub struct WikiIndex {
    /// `(id, title)`, the first created first.
    pub titles: Vec<(String, String)>,
    /// Normalized title to the first note with it, as an index into
    /// `titles`.
    by_title: HashMap<String, usize>,
}

impl WikiIndex {
    /// The index of `titles`, given first created first: a title shared by
    /// several notes names the first.
    pub fn new(titles: Vec<(String, String)>) -> Self {
        let mut by_title = HashMap::with_capacity(titles.len());
        for (ix, (_, title)) in titles.iter().enumerate() {
            let key = normalize_title(title);
            if !key.is_empty() {
                by_title.entry(key).or_insert(ix);
            }
        }
        WikiIndex { titles, by_title }
    }

    /// The `(id, title)` of the note `title` names.
    pub fn resolve(&self, title: &str) -> Option<&(String, String)> {
        self.by_title
            .get(&normalize_title(title))
            .map(|&ix| &self.titles[ix])
    }
}

/// Text as a link target can carry it: everything but letters, digits and
/// a few safe marks percent-encoded. Pure.
pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Text from a link target, percent-decoded; bytes that don't decode stay
/// as they are, and so does text that isn't UTF-8 once decoded (`None`).
/// Pure.
pub fn percent_decode(encoded: &str) -> Option<String> {
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = encoded.get(i + 1..i + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            out.push(byte);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// `kanso-new:<title>`, with everything but letters, digits and a few safe
/// marks percent-encoded, so the title survives as a link target.
pub fn new_note_url(title: &str) -> String {
    format!("{NEW_NOTE_SCHEME}{}", percent_encode(title))
}

/// The title a `kanso-new:` link asks to create.
pub fn new_note_title(url: &str) -> Option<String> {
    let title = percent_decode(url.strip_prefix(NEW_NOTE_SCHEME)?)?;
    let title = title.trim();
    (!title.is_empty()).then(|| title.to_owned())
}

/// The note's Markdown with each wiki-link outside code written as an
/// ordinary link, for the preview: resolved ones to their note, others to a
/// `kanso-new:` link. Text without wiki-links comes back as it is
/// (borrowed). Pure.
pub fn rewrite_for_preview<'a>(text: &'a str, index: &WikiIndex) -> Cow<'a, str> {
    rewrite(text, index, false)
}

/// [`rewrite_for_preview`] for the phone's page: a wiki-link to a heading
/// (`[[Title#Heading]]`) keeps it, as `…#Heading` (percent-encoded) after the
/// note's address, so the app can open the note at the heading. Pure.
pub fn rewrite_for_app<'a>(text: &'a str, index: &WikiIndex) -> Cow<'a, str> {
    rewrite(text, index, true)
}

fn rewrite<'a>(text: &'a str, index: &WikiIndex, keep_heading: bool) -> Cow<'a, str> {
    if !text.contains("[[") {
        return Cow::Borrowed(text);
    }
    let links = wiki_links(text);
    if links.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + links.len() * 32);
    let mut last = 0;
    for link in links {
        out.push_str(&text[last..link.range.start]);
        let label = escape_label(&link.label());
        let target = match index.resolve(&link.title) {
            Some((id, _)) => format!(
                "{NOTE_LINK_PREFIX}{}{}",
                id.strip_prefix("note:").unwrap_or(id),
                match &link.heading {
                    Some(heading) if keep_heading => format!("#{}", percent_encode(heading)),
                    _ => String::new(),
                }
            ),
            None => new_note_url(&link.title),
        };
        out.push('[');
        out.push_str(&label);
        out.push_str("](");
        out.push_str(&target);
        out.push(')');
        last = link.range.end;
    }
    out.push_str(&text[last..]);
    Cow::Owned(out)
}

/// Link text that stays one link: brackets and backslashes escaped.
fn escape_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if matches!(c, '[' | ']' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

// MARK: Autocomplete

/// What `[[` autocomplete is completing, from the text of the caret's line
/// up to the caret: where the `[[` starts in it and what was typed after.
/// `None` once the brackets are closed, the line has none, or the typing
/// can't be a title (a bracket inside). Pure.
pub fn completion_query(before_caret: &str) -> Option<(usize, &str)> {
    let start = before_caret.rfind("[[")?;
    // `[[[`: the last two brackets open it.
    let query = &before_caret[start + 2..];
    if query.contains(['[', ']']) || query.contains('|') {
        return None;
    }
    if start > 0 && before_caret.as_bytes()[start - 1] == b'!' {
        return None;
    }
    if query.chars().count() > 80 {
        return None;
    }
    Some((start, query))
}

/// Kanso's own link to note `id` titled `title`, `[Title](<ID link>)`:
/// what autocomplete writes, which names the note by ID. Pure.
pub fn id_link(id: &str, title: &str) -> String {
    let suffix = id.strip_prefix("note:").unwrap_or(id);
    format!("[{}]({NOTE_LINK_PREFIX}{suffix})", link_text(title))
}

/// Link text for `title` in `[text](…)`: brackets escaped, and "Untitled"
/// for a note without one.
fn link_text(title: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        "Untitled".to_owned()
    } else {
        escape_label(title)
    }
}

/// The titles autocomplete offers for `query`, best first, at most
/// `limit`: every title the query fits (fuzzily), or with nothing typed the
/// most recently created. Pure.
pub fn completion_candidates<'a>(
    index: &'a WikiIndex,
    query: &str,
    limit: usize,
) -> Vec<&'a (String, String)> {
    let query = query.trim();
    if query.is_empty() {
        return index.titles.iter().rev().take(limit).collect();
    }
    let mut scored: Vec<(i64, usize)> = index
        .titles
        .iter()
        .enumerate()
        .filter_map(|(ix, (_, title))| crate::fuzzy::score(query, title).map(|s| (s, ix)))
        .collect();
    // Best score first; among equals, the newer note.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, ix)| &index.titles[ix])
        .collect()
}

/// The start of `title` that `query` spells, ignoring case, for the list
/// to highlight; empty when the title doesn't start with it (a fuzzy match
/// further in). The list highlights only a prefix. Pure.
pub fn matched_prefix<'a>(title: &'a str, query: &str) -> &'a str {
    let query = query.trim();
    let mut end = 0;
    let mut wanted = query.chars().flat_map(char::to_lowercase);
    let mut pending = wanted.next();
    for (ix, ch) in title.char_indices() {
        let Some(want) = pending else { break };
        let mut lower = ch.to_lowercase();
        if lower.next() != Some(want) || lower.next().is_some() {
            return "";
        }
        end = ix + ch.len_utf8();
        pending = wanted.next();
    }
    if pending.is_some() { "" } else { &title[..end] }
}

/// What `[[` autocomplete offers at the caret: the part of the line a
/// choice replaces and the choices, best first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiChoices {
    /// Byte range of the line from the `[[` to the caret, and past it over
    /// the `]]` the pairing closed after the caret.
    pub replace: Range<usize>,
    pub items: Vec<WikiChoice>,
}

/// One choice of `[[` autocomplete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WikiChoice {
    /// What the list shows.
    pub label: String,
    /// What replaces [`WikiChoices::replace`] when it is picked.
    pub insert: String,
    /// The part of the label the list highlights: the start of the title
    /// the query spells, or nothing.
    pub highlight: String,
    /// Whether this is the entry that writes `[[typed]]` for a note that
    /// doesn't exist yet.
    pub new_note: bool,
}

/// `[[` autocomplete on `line` (without its line break) with the caret at
/// byte `caret`: notes by title, inserted as `[Title](<ID link>)`. When no
/// title matches what was typed exactly, the last entry writes `[[typed]]`,
/// which the preview offers to create. `None` when the caret isn't in an
/// open `[[…`, or it is in a code span, where `[[` is text. Pure.
pub fn wiki_choices(line: &str, caret: usize, index: &WikiIndex) -> Option<WikiChoices> {
    let mut caret = caret.min(line.len());
    while !line.is_char_boundary(caret) {
        caret -= 1;
    }
    // Only the end of a long line can hold the query.
    let mut from = caret.saturating_sub(LOOK_BACK);
    while !line.is_char_boundary(from) {
        from += 1;
    }
    let (start, query) = completion_query(&line[from..caret])?;
    let start = from + start;
    if code_ranges(line)
        .iter()
        .any(|r| r.start <= start && start < r.end)
    {
        return None;
    }
    // Brackets the pairing closed after the caret go too.
    let end = if line.get(caret..caret + 2) == Some("]]") {
        caret + 2
    } else {
        caret
    };
    let mut items: Vec<WikiChoice> = completion_candidates(index, query, COMPLETION_LIMIT)
        .into_iter()
        .map(|(id, title)| {
            let shown = if title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                title.clone()
            };
            WikiChoice {
                highlight: matched_prefix(&shown, query).to_owned(),
                label: shown,
                insert: id_link(id, title),
                new_note: false,
            }
        })
        .collect();
    let typed = query.trim();
    if !typed.is_empty() && index.resolve(typed).is_none() {
        items.push(WikiChoice {
            label: format!("New note “{typed}”"),
            insert: format!("[[{typed}]]"),
            highlight: String::new(),
            new_note: true,
        });
    }
    Some(WikiChoices {
        replace: start..end,
        items,
    })
}

// MARK: Copying a link

/// The wiki-link naming a note titled `title`, or `None` when the title
/// can't be written as one (it is empty, or has a bracket, `#`, `|` or a
/// line break in it). Pure.
pub fn wiki_link(title: &str) -> Option<String> {
    let title = title.trim();
    (!title.contains(['#', '|']) && parse_target(title).is_some()).then(|| format!("[[{title}]]"))
}

/// What to copy for note `id` titled `title`: its wiki-link, unless
/// `kanso` asks for Kanso's own link, or the title can't be a wiki-link,
/// or the title names an older note first (`resolves_to` is the note the
/// title resolves to). Says whether the wiki-link was given way. Pure.
pub fn link_for(id: &str, title: &str, kanso: bool, resolves_to: Option<&str>) -> (String, bool) {
    let wiki = (!kanso)
        .then(|| wiki_link(title))
        .flatten()
        .filter(|_| resolves_to.is_none_or(|other| other == id));
    match wiki {
        Some(link) => (link, false),
        None => (id_link(id, title), !kanso),
    }
}

/// What to say after copying `title`'s link: `instead` is the second part
/// of [`link_for`]'s answer. Pure.
pub fn copy_message(title: &str, instead: bool) -> &'static str {
    if !instead {
        "Link copied."
    } else if wiki_link(title).is_none() {
        "Copied a Kanso link: the title can't be a [[wiki-link]]."
    } else {
        "Copied a Kanso link: an older note has the same title."
    }
}

/// The note ID an in-app link points at.
pub fn linked_note(url: &str) -> Option<String> {
    let suffix = url.strip_prefix(NOTE_LINK_PREFIX).or_else(|| {
        url.strip_prefix(LINK_SCHEME)
            .and_then(|rest| rest.strip_prefix("note:"))
    })?;
    let suffix = suffix.split(['?', '#', '/']).next()?;
    (!suffix.is_empty()).then(|| format!("note:{suffix}"))
}

// MARK: The link at the caret

/// What a link points at, as the editor opens it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkAtTarget {
    /// A note by ID (`note:…`), from a Kanso link.
    Note { id: String },
    /// A note by title: a wiki-link, or a link to a note that doesn't exist
    /// yet. Resolving it, or creating it, is the caller's.
    Wiki { title: String },
    /// A web address (or any other address the system opens).
    Url { url: String },
}

/// A link on a line and what it points at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkAt {
    /// The bytes of the whole link in the line: `[[…]]`, `[text](…)` with
    /// its `!` when an image, or the bare address.
    pub range: Range<usize>,
    pub target: LinkAtTarget,
}

/// The link the caret at byte `caret` is in or touches on `line` (without
/// its line break): a wiki-link, a Markdown link or image, an `<address>`
/// or a bare `http(s)://` address, outside code. Links don't nest, and
/// where two touch at the caret the first wins. Pure.
pub fn link_at(line: &str, caret: usize) -> Option<LinkAt> {
    let mut caret = caret.min(line.len());
    while !line.is_char_boundary(caret) {
        caret -= 1;
    }
    let code = code_ranges(line);
    let mut found: Vec<LinkAt> = wiki_links(line)
        .into_iter()
        .map(|link| LinkAt {
            range: link.range,
            target: LinkAtTarget::Wiki { title: link.title },
        })
        .collect();
    found.extend(markdown_links(line, &code));
    found.sort_by_key(|link| link.range.start);
    // Addresses written out, where no link already is.
    let taken: Vec<Range<usize>> = found.iter().map(|link| link.range.clone()).collect();
    for range in bare_addresses(line) {
        let inside = |r: &Range<usize>| r.start < range.end && range.start < r.end;
        if !taken.iter().any(inside) && !code.iter().any(inside) {
            found.push(LinkAt {
                target: LinkAtTarget::Url {
                    url: line[range.clone()].to_owned(),
                },
                range,
            });
        }
    }
    found.sort_by_key(|link| link.range.start);
    found
        .into_iter()
        .find(|link| link.range.start <= caret && caret <= link.range.end)
}

/// The `[text](destination)` links and images in `line` outside `code`.
fn markdown_links(line: &str, code: &[Range<usize>]) -> Vec<LinkAt> {
    let bytes = line.as_bytes();
    let in_code = |at: usize| code.iter().any(|r| r.start <= at && at < r.end);
    let mut links = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] != b'[' || in_code(i) {
            i += 1;
            continue;
        }
        // The text, up to its matching bracket.
        let mut depth = 0;
        let mut j = i;
        let mut close = None;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' => j += 1,
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(j);
                        break;
                    }
                }
                _ => {}
            }
            j += 1;
        }
        let Some(close) = close else { break };
        if bytes.get(close + 1) != Some(&b'(') {
            i += 1;
            continue;
        }
        // The destination, up to the matching parenthesis.
        let mut depth = 0;
        let mut k = close + 1;
        let mut end = None;
        while k < bytes.len() {
            match bytes[k] {
                b'\\' => k += 1,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(k);
                        break;
                    }
                }
                _ => {}
            }
            k += 1;
        }
        let Some(end) = end else {
            i += 1;
            continue;
        };
        let inner = line[close + 2..end].trim();
        // `<destination>`, or the destination up to a space and a title.
        let destination = match inner.strip_prefix('<') {
            Some(rest) => rest.split('>').next().unwrap_or(""),
            None => inner.split_whitespace().next().unwrap_or(""),
        };
        let start = if i > 0 && bytes[i - 1] == b'!' && !in_code(i - 1) {
            i - 1
        } else {
            i
        };
        if let Some(target) = destination_target(destination) {
            links.push(LinkAt {
                range: start..end + 1,
                target,
            });
        }
        i = end + 1;
    }
    links
}

/// What a link destination points at; `None` for an empty one or one that
/// is only a placeholder.
fn destination_target(destination: &str) -> Option<LinkAtTarget> {
    if destination.is_empty() || destination.starts_with('#') {
        return None;
    }
    if let Some(id) = linked_note(destination) {
        return Some(LinkAtTarget::Note { id });
    }
    if let Some(title) = new_note_title(destination) {
        return Some(LinkAtTarget::Wiki { title });
    }
    // The placeholder a new link starts with.
    if matches!(destination, "http://" | "https://") {
        return None;
    }
    Some(LinkAtTarget::Url {
        url: destination.to_owned(),
    })
}

/// The `http://` and `https://` addresses in `line`, up to a space or a
/// closing bracket or quote, without the punctuation a sentence puts after
/// one.
fn bare_addresses(line: &str) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    let mut from = 0;
    while from < line.len() {
        let rest = &line[from..];
        let Some(at) = ["https://", "http://"]
            .iter()
            .filter_map(|scheme| rest.find(scheme))
            .min()
        else {
            break;
        };
        let start = from + at;
        let tail = &line[start..];
        let len = tail
            .find(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'' | '`'))
            .unwrap_or(tail.len());
        let address = tail[..len].trim_end_matches(['.', ',', ';', ':', '!', '?', ')', ']']);
        let scheme = if address.starts_with("https://") {
            8
        } else {
            7
        };
        if address.len() > scheme {
            found.push(start..start + address.len());
        }
        from = start + len.max(1);
    }
    found
}

// MARK: Renaming

/// After note `id` was renamed from `old` to `new`: how many wiki-links
/// still use the old title, and in how many notes, when other notes name it
/// and the workspace should offer to update them. `None` when the titles
/// are the same once normalized, either is blank, an older note has the old
/// title (so the links never named this note), or nothing links to it. The
/// store leaves out the trash, and the sealed text of locked notes holds no
/// links to count. Pure but for reading `store`.
pub fn relink_offer(store: &Store, id: &str, old: &str, new: &str) -> Option<(usize, usize)> {
    if old.trim().is_empty()
        || new.trim().is_empty()
        || normalize_title(old) == normalize_title(new)
    {
        return None;
    }
    // The old title only named this note if no older note has it.
    if let Ok(Some((other, _))) = store.resolve_title(old)
        && other != id
    {
        return None;
    }
    match store.wiki_link_count(old, id) {
        Ok((links, notes)) if links > 0 => Some((links, notes)),
        _ => None,
    }
}

/// What the offer to update links after a rename says. Pure.
pub fn relink_message(old: &str, links: usize, notes: usize) -> String {
    let links_word = if links == 1 { "link" } else { "links" };
    let notes_part = if notes == 1 {
        "1 note".to_owned()
    } else {
        format!("{notes} notes")
    };
    format!("{links} {links_word} in {notes_part} still use the old title “{old}”.")
}

/// The offer's button: "Update 1 link", "Update 3 links". Pure.
pub fn relink_label(links: usize) -> String {
    if links == 1 {
        "Update 1 link".to_owned()
    } else {
        format!("Update {links} links")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use kanso_core::links::note_links;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    fn index() -> WikiIndex {
        WikiIndex::new(vec![
            ("note:a".into(), "Project Plan".into()),
            ("note:b".into(), "Budget".into()),
            ("note:c".into(), "project plan".into()),
        ])
    }

    #[test]
    fn only_a_spelled_prefix_is_highlighted() {
        assert_eq!(matched_prefix("Sync design", "sy"), "Sy");
        assert_eq!(matched_prefix("MCP support model", "s"), "");
        assert_eq!(matched_prefix("Café crème", "CAFÉ"), "Café");
        assert_eq!(matched_prefix("Ab", "abc"), "");
        assert_eq!(matched_prefix("Ab", ""), "");
    }

    #[test]
    fn the_first_created_note_wins_a_shared_title() {
        assert_eq!(index().resolve("PROJECT  plan").unwrap().0, "note:a");
        assert!(index().resolve("Nope").is_none());
    }

    #[test]
    fn the_preview_gets_ordinary_links() {
        let text = "See [[project plan#Goals|the plan]], [[Missing Note]] and `[[Budget]]`.";
        assert_eq!(
            rewrite_for_preview(text, &index()).as_ref(),
            format!(
                "See [the plan]({NOTE_LINK_PREFIX}a), [Missing Note](kanso-new:Missing%20Note) and `[[Budget]]`."
            )
        );
        let plain = "No links here";
        assert!(matches!(
            rewrite_for_preview(plain, &index()),
            Cow::Borrowed("No links here")
        ));
    }

    #[test]
    fn completion_queries() {
        assert_eq!(completion_query("see [[Pro"), Some((4, "Pro")));
        assert_eq!(completion_query("[["), Some((0, "")));
        assert_eq!(completion_query("[[done]] x"), None);
        assert_eq!(completion_query("![[img"), None);
        assert_eq!(completion_query("plain"), None);
        let index = index();
        let found: Vec<&str> = completion_candidates(&index, "bud", 5)
            .into_iter()
            .map(|(id, _)| id.as_str())
            .collect();
        assert_eq!(found, ["note:b"]);
        assert_eq!(completion_candidates(&index, "", 2).len(), 2);
    }

    #[test]
    fn choices_replace_the_brackets_and_offer_a_new_note() {
        let index = index();
        let line = "see [[bud]] now";
        let choices = wiki_choices(line, 9, &index).unwrap();
        // The `]]` the pairing closed goes with it.
        assert_eq!(&line[choices.replace.clone()], "[[bud]]");
        assert_eq!(choices.items[0].label, "Budget");
        assert_eq!(choices.items[0].insert, id_link("note:b", "Budget"));
        assert_eq!(choices.items[0].highlight, "Bud");
        let last = choices.items.last().unwrap();
        assert!(last.new_note);
        assert_eq!(last.label, "New note “bud”");
        assert_eq!(last.insert, "[[bud]]");
        // A title typed in full needs no new note.
        let choices = wiki_choices("[[Budget", 8, &index).unwrap();
        assert!(choices.items.iter().all(|c| !c.new_note));
        // Code is text; so is a closed link.
        assert_eq!(wiki_choices("`a [[bu`", 7, &index), None);
        assert_eq!(wiki_choices("[[Budget]] x", 12, &index), None);
        // An untitled note is offered as "Untitled".
        let blank = WikiIndex::new(vec![("note:z".into(), String::new())]);
        let choices = wiki_choices("[[", 2, &blank).unwrap();
        assert_eq!(choices.items[0].label, "Untitled");
        assert_eq!(
            choices.items[0].insert,
            format!("[Untitled]({NOTE_LINK_PREFIX}z)")
        );
    }

    #[test]
    fn link_at_examples() {
        let line = "see [[Plan|the plan]], [docs](https://a.b/c \"t\") and https://x.y/z.";
        let at = |caret| link_at(line, caret).map(|l| (line[l.range].to_owned(), l.target));
        assert_eq!(
            at(8),
            Some((
                "[[Plan|the plan]]".into(),
                LinkAtTarget::Wiki {
                    title: "Plan".into()
                }
            ))
        );
        assert_eq!(
            at(26),
            Some((
                "[docs](https://a.b/c \"t\")".into(),
                LinkAtTarget::Url {
                    url: "https://a.b/c".into()
                }
            ))
        );
        // A sentence's full stop is not the address's.
        assert_eq!(
            at(line.len() - 3),
            Some((
                "https://x.y/z".into(),
                LinkAtTarget::Url {
                    url: "https://x.y/z".into()
                }
            ))
        );
        assert_eq!(at(2), None);
        assert_eq!(link_at("`[[a]]`", 3), None);
        // A new link's placeholder is not an address yet.
        assert_eq!(link_at("[x](https://)", 4), None);
        // An image is a link to its address.
        assert_eq!(
            link_at("![i](https://a.b/i.png)", 5).map(|l| l.range),
            Some(0..23)
        );
    }

    #[test]
    fn a_plain_title_is_a_wiki_link() {
        assert_eq!(wiki_link("Plans"), Some("[[Plans]]".into()));
        assert_eq!(wiki_link("  Weekly plan "), Some("[[Weekly plan]]".into()));
    }

    #[test]
    fn titles_a_wiki_link_cant_hold_have_none() {
        for title in [
            "",
            "  ",
            "C# notes",
            "a | b",
            "[draft]",
            "x]]y",
            "two\nlines",
        ] {
            assert_eq!(wiki_link(title), None, "{title:?}");
        }
    }

    #[test]
    fn the_kanso_link_names_the_note_by_id() {
        let (link, instead) = link_for("note:abc", "Plans", true, None);
        assert_eq!(link, format!("[Plans]({NOTE_LINK_PREFIX}abc)"));
        assert!(!instead);
    }

    #[test]
    fn a_title_an_older_note_has_copies_the_kanso_link() {
        let (link, instead) = link_for("note:new", "Plans", false, Some("note:old"));
        assert_eq!(link, format!("[Plans]({NOTE_LINK_PREFIX}new)"));
        assert!(instead);
        assert_eq!(
            copy_message("Plans", instead),
            "Copied a Kanso link: an older note has the same title."
        );
        let (link, instead) = link_for("note:old", "Plans", false, Some("note:old"));
        assert_eq!(link, "[[Plans]]");
        assert!(!instead);
        assert_eq!(copy_message("Plans", instead), "Link copied.");
    }

    #[test]
    fn an_untitled_note_copies_the_kanso_link() {
        let (link, instead) = link_for("note:abc", "", false, None);
        assert_eq!(link, format!("[Untitled]({NOTE_LINK_PREFIX}abc)"));
        assert!(instead);
        assert_eq!(
            copy_message("", instead),
            "Copied a Kanso link: the title can't be a [[wiki-link]]."
        );
    }

    #[test]
    fn linked_notes_are_found_by_either_prefix() {
        assert_eq!(
            linked_note(&format!("{NOTE_LINK_PREFIX}abc?x=1")),
            Some("note:abc".into())
        );
        assert_eq!(
            linked_note(&format!("{LINK_SCHEME}note:abc")),
            Some("note:abc".into())
        );
        assert_eq!(linked_note(NOTE_LINK_PREFIX), None);
        assert_eq!(linked_note("https://example.com/note/abc"), None);
    }

    #[test]
    fn the_relink_words_count() {
        assert_eq!(relink_label(1), "Update 1 link");
        assert_eq!(relink_label(2), "Update 2 links");
        assert_eq!(
            relink_message("Old", 1, 1),
            "1 link in 1 note still use the old title “Old”."
        );
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn put_note(store: &Store, title: &str, body: &str) -> String {
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": title, "body": body}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// The offer counts the links other notes still have to the old title,
    /// and says nothing when the title only changed in spacing or case,
    /// when either is blank, when an older note has the old title, or when
    /// nothing links to it.
    #[test]
    fn a_rename_offers_the_links_that_still_use_the_old_title() {
        let (_dir, store) = store();
        let renamed = put_note(&store, "New", "");
        put_note(&store, "A", "see [[Old]] and [[old|again]]");
        put_note(&store, "B", "also [[Old]], not [[Other]]");
        assert_eq!(relink_offer(&store, &renamed, "Old", "New"), Some((3, 2)));
        assert_eq!(relink_offer(&store, &renamed, "Old", " new "), Some((3, 2)));
        // The same title, spaced or cased differently.
        assert_eq!(relink_offer(&store, &renamed, "Old", " old  "), None);
        assert_eq!(relink_offer(&store, &renamed, " ", "New"), None);
        assert_eq!(relink_offer(&store, &renamed, "Old", "  "), None);
        // Nothing links to it.
        assert_eq!(relink_offer(&store, &renamed, "Nobody", "New"), None);
        // The renamed note's own links don't count.
        let own = put_note(&store, "Own", "[[Mine]]");
        assert_eq!(relink_offer(&store, &own, "Mine", "Own"), None);
    }

    #[test]
    fn an_older_note_with_the_old_title_keeps_its_links() {
        let (_dir, store) = store();
        put_note(&store, "Old", "");
        let renamed = put_note(&store, "New", "");
        put_note(&store, "A", "see [[Old]]");
        assert_eq!(relink_offer(&store, &renamed, "Old", "New"), None);
    }

    /// A locked note's links are sealed with its text: the offer doesn't
    /// count them, and so never says a locked note links anywhere.
    #[test]
    fn the_offer_leaves_locked_notes_out() {
        use kanso_sync::KdfParams;
        use kanso_sync::lock::{Envelope, LockKey, LockPassword};

        let (_dir, store) = store();
        let renamed = put_note(&store, "New", "");
        put_note(&store, "Open", "see [[Old]]");
        let locked = put_note(&store, "Locked", "see [[Old]] and [[Old]]");
        assert_eq!(relink_offer(&store, &renamed, "Old", "New"), Some((3, 2)));

        let key = LockKey::create(
            &LockPassword::new("a lock password").unwrap(),
            KdfParams::TEST,
        )
        .unwrap();
        let mut note = store.note(&locked).unwrap().unwrap();
        Envelope::store(Some(&key.seal(&locked, &note.body)), &mut note.extra);
        note.body = String::new();
        store
            .lock_note(serde_json::to_value(&note).unwrap())
            .unwrap();
        assert!(store.note(&locked).unwrap().unwrap().is_locked());
        assert_eq!(relink_offer(&store, &renamed, "Old", "New"), Some((1, 1)));
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The highlight is always the title's own start, as long as the
        /// query or nothing.
        #[test]
        fn the_highlight_is_a_prefix(title in "\\PC{0,12}", query in "\\PC{0,4}") {
            let got = matched_prefix(&title, &query);
            prop_assert!(title.starts_with(got));
            prop_assert!(got.is_empty() || got.to_lowercase() == query.trim().to_lowercase());
        }

        /// Round trip: any title survives the `kanso-new:` link.
        #[test]
        fn prop_new_note_urls_round_trip(title in "[^\\s]([^\\n]{0,30}[^\\s])?") {
            let url = new_note_url(&title);
            prop_assert!(!url[NEW_NOTE_SCHEME.len()..].contains([' ', '(', ')', '[', ']', '<', '>']));
            prop_assert_eq!(new_note_title(&url), Some(title));
        }

        /// Invariant: rewriting leaves no wiki-link outside code, keeps the
        /// text around them, and every link it writes points at a note the
        /// index resolves or at `kanso-new:`.
        #[test]
        fn prop_rewritten_text_has_no_wiki_links(parts in prop::collection::vec(prop_oneof!["[a-z ]{0,6}", Just("[[Budget]]".to_owned()), Just("[[Nope|x]]".to_owned()), Just("`[[Budget]]`".to_owned())], 0..8)) {
            let text = parts.concat();
            let out = rewrite_for_preview(&text, &index());
            prop_assert!(wiki_links(&out).is_empty());
            let code_kept = parts.iter().filter(|p| p.starts_with('`')).count();
            prop_assert_eq!(out.matches("`[[Budget]]`").count(), code_kept);
            let resolved = parts.iter().filter(|p| p.as_str() == "[[Budget]]").count();
            prop_assert_eq!(out.matches(&format!("]({NOTE_LINK_PREFIX}b)")).count(), resolved);
        }

        /// The query is what follows the last `[[` on the line, when it can
        /// still be a title.
        #[test]
        fn prop_completion_query_is_the_text_after_the_brackets(pre in "[a-z ]{0,8}", typed in "[A-Za-z0-9 ]{0,12}") {
            let line = format!("{pre}[[{typed}");
            prop_assert_eq!(completion_query(&line), Some((pre.len(), typed.as_str())));
            let closed = format!("{line}]]");
            prop_assert_eq!(completion_query(&closed), None);
        }

        /// The choices replace exactly `[[typed`, and the `]]` after the
        /// caret when it is there; each link they insert reads back, through
        /// the note links, as its note; the entry for a new note is there
        /// exactly when something was typed that names no note; and a
        /// caret in code gets none.
        #[test]
        fn prop_choices_replace_what_was_typed(
            pre in "[a-z ]{0,8}",
            typed in "[A-Za-z0-9é ]{0,12}",
            closed in any::<bool>(),
            post in "[a-z ]{0,6}",
            titles in prop::collection::vec("[A-Za-z0-9é ]{0,10}", 0..8),
        ) {
            let index = WikiIndex::new(
                titles.iter().enumerate().map(|(i, t)| (format!("note:n{i}"), t.clone())).collect(),
            );
            let tail = if closed { format!("]]{post}") } else { post.clone() };
            let line = format!("{pre}[[{typed}{tail}");
            let caret = pre.len() + 2 + typed.len();
            let got = wiki_choices(&line, caret, &index)
                .ok_or_else(|| TestCaseError::fail(format!("no choices for {line:?} at {caret}")))?;
            let want_end = caret + if closed { 2 } else { 0 };
            prop_assert_eq!(got.replace.clone(), pre.len()..want_end);
            prop_assert!(got.items.len() <= COMPLETION_LIMIT + 1);
            for item in got.items.iter().filter(|i| !i.new_note) {
                let links = note_links(&item.insert);
                prop_assert_eq!(links.len(), 1, "{:?}", item.insert);
                let target = format!("({NOTE_LINK_PREFIX}n");
                prop_assert!(item.insert.contains(&target));
            }
            let typed_title = typed.trim();
            let unresolved = !typed_title.is_empty() && index.resolve(typed_title).is_none();
            let news: Vec<_> = got.items.iter().filter(|i| i.new_note).collect();
            prop_assert_eq!(news.len(), usize::from(unresolved));
            if unresolved {
                prop_assert!(got.items.last().unwrap().new_note);
                prop_assert_eq!(&news[0].insert, &format!("[[{typed_title}]]"));
            }
            // In a code span, `[[` is text.
            let coded = format!("`{pre}[[{typed}`");
            prop_assert_eq!(wiki_choices(&coded, coded.len() - 1, &index), None);
        }

        /// The caret may be anywhere, even inside a character: no panic,
        /// and what is replaced is a slice of the line.
        #[test]
        fn prop_choices_never_split_a_character(line in "\\PC{0,30}", caret in 0usize..80) {
            if let Some(got) = wiki_choices(&line, caret, &index()) {
                prop_assert!(line.get(got.replace.clone()).is_some());
            }
        }

        /// A wiki-link copied for a title reads back, from a note's text,
        /// as one link to that title and nothing else.
        #[test]
        fn a_copied_wiki_link_reads_back_as_its_title(
            title in "[ a-zA-Z0-9#|\\[\\]\\-_.,'!?é]{0,24}",
            before in "[ a-z.]{0,6}",
        ) {
            if let Some(link) = wiki_link(&title) {
                let body = format!("{before}{link} after");
                let links = wiki_links(&body);
                prop_assert_eq!(links.len(), 1);
                prop_assert_eq!(&links[0].title, title.trim());
                prop_assert_eq!(links[0].heading.as_ref(), None);
                prop_assert_eq!(links[0].alias.as_ref(), None);
            } else {
                prop_assert!(
                    title.trim().is_empty() || title.contains(['#', '|', '[', ']']),
                    "a plain title gave no link: {:?}", title
                );
            }
        }

        /// `link_at` on a line made of known pieces: a caret on or at the
        /// edge of a link finds exactly that link, with its target, and
        /// anywhere else finds nothing.
        #[test]
        fn prop_link_at_finds_the_piece_under_the_caret(
            pieces in prop::collection::vec(
                prop_oneof![
                    "[a-z]{1,6}".prop_map(|t| (t, None)),
                    "[A-Za-z]{1,6}( [A-Za-z]{1,5})?".prop_map(|t: String| (
                        format!("[[{t}]]"),
                        Some(LinkAtTarget::Wiki { title: t }),
                    )),
                    ("[A-Za-z]{1,5}", "[a-z0-9]{1,6}").prop_map(|(t, id)| (
                        format!("[{t}]({NOTE_LINK_PREFIX}{id})"),
                        Some(LinkAtTarget::Note { id: format!("note:{id}") }),
                    )),
                    "[A-Za-z]{1,6}".prop_map(|t| (
                        format!("[{t}]({})", new_note_url(&t)),
                        Some(LinkAtTarget::Wiki { title: t }),
                    )),
                    ("[A-Za-z]{1,5}", "[a-z]{1,6}").prop_map(|(t, host)| {
                        let url = format!("https://{host}.example/p");
                        (format!("[{t}]({url})"), Some(LinkAtTarget::Url { url }))
                    }),
                    "[a-z]{1,6}".prop_map(|host| {
                        let url = format!("http://{host}.example/q?x=1");
                        (url.clone(), Some(LinkAtTarget::Url { url }))
                    }),
                ],
                1..6,
            ),
        ) {
            let mut line = String::new();
            let mut spans = Vec::new();
            for (text, target) in &pieces {
                if !line.is_empty() {
                    line.push(' ');
                }
                let start = line.len();
                line.push_str(text);
                if let Some(target) = target {
                    spans.push((start..line.len(), target.clone()));
                }
            }
            for caret in 0..=line.len() {
                let found = link_at(&line, caret);
                let expected = spans.iter().find(|(range, _)| range.start <= caret && caret <= range.end);
                match (found, expected) {
                    (Some(found), Some((range, target))) => {
                        prop_assert_eq!(&found.range, range);
                        prop_assert_eq!(&found.target, target);
                    }
                    (None, None) => {}
                    (found, expected) => prop_assert!(false, "{caret} in {line:?}: {found:?} vs {expected:?}"),
                }
            }
        }

        /// Whatever the line and the caret, a link found has its range in
        /// the line, on character boundaries, around the caret; and none is
        /// found in code.
        #[test]
        fn prop_link_at_is_in_bounds_and_never_in_code(line in "[a-z\\[\\]()!<>:/. `é日🙂]{0,40}", caret in 0usize..60) {
            if let Some(found) = link_at(&line, caret) {
                prop_assert!(found.range.start <= found.range.end && found.range.end <= line.len());
                prop_assert!(line.is_char_boundary(found.range.start) && line.is_char_boundary(found.range.end));
                let mut caret = caret.min(line.len());
                while !line.is_char_boundary(caret) { caret -= 1; }
                prop_assert!(found.range.start <= caret && caret <= found.range.end);
            }
            let fenced = format!("`{line}`");
            let at = fenced.len() / 2;
            if !line.contains('`') {
                prop_assert_eq!(link_at(&fenced, at), None);
            }
        }

        /// The Kanso link points at the note's ID, whatever its title.
        #[test]
        fn the_kanso_link_points_at_the_id(
            id in "[a-zA-Z0-9_-]{1,16}",
            title in "\\PC{0,24}",
        ) {
            let (link, _) = link_for(&format!("note:{id}"), &title, true, None);
            let target = link.rsplit_once("](").map(|(_, rest)| rest.trim_end_matches(')'));
            prop_assert_eq!(
                target.and_then(linked_note),
                Some(format!("note:{id}"))
            );
        }

        /// The offer's words agree in number with what it counts.
        #[test]
        fn prop_relink_words_agree_in_number(links in 1usize..20, notes in 1usize..20) {
            let message = relink_message("Old", links, notes);
            let label = relink_label(links);
            prop_assert_eq!(message.contains(" link in ") || message.contains(" links in "), true);
            prop_assert_eq!(links == 1, label.ends_with(" link"));
            prop_assert_eq!(notes == 1, message.contains(" 1 note "));
        }
    }
}
