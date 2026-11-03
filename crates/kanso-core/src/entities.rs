//! People and projects: ordinary notes marked `kanso: person` or
//! `kanso: project` in their front matter, with optional `aliases`.
//!
//! This module is the pure part: reading the mark and the names off a note,
//! finding `@name` mentions, and finding every mention of a set of names in
//! a body. The store keeps the results in derived tables.

use std::collections::{HashMap, HashSet};

use crate::links::{
    LinkTarget, code_ranges, covered_ranges, mention_lines, normalize_title, note_links,
};
use crate::properties::{self, Value};

/// The front matter key that marks a note as a page for an entity.
pub const KIND_KEY: &str = "kanso";
/// The front matter key holding the other names.
pub const ALIASES_KEY: &str = "aliases";
/// A name shorter than this is never found as plain words.
pub const MIN_WORD_NAME: usize = 3;

/// What a page is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EntityKind {
    Person,
    Project,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Person => "person",
            EntityKind::Project => "project",
        }
    }

    pub fn parse(text: &str) -> Option<EntityKind> {
        match text.trim().to_lowercase().as_str() {
            "person" => Some(EntityKind::Person),
            "project" => Some(EntityKind::Project),
            _ => None,
        }
    }
}

/// How a note mentions an entity. Ordered by how sure the mention is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum How {
    Link,
    At,
    Word,
}

impl How {
    pub fn as_str(self) -> &'static str {
        match self {
            How::Link => "link",
            How::At => "at",
            How::Word => "word",
        }
    }

    pub fn parse(text: &str) -> How {
        match text {
            "link" => How::Link,
            "at" => How::At,
            _ => How::Word,
        }
    }
}

/// The kind a note's front matter marks, if any.
pub fn kind_of(body: &str) -> Option<EntityKind> {
    let props = properties::parse(body);
    match properties::get(&props, KIND_KEY)? {
        Value::Text(text) => EntityKind::parse(text),
        _ => None,
    }
}

/// The aliases a note's front matter lists, as written.
pub fn aliases_of(body: &str) -> Vec<String> {
    let props = properties::parse(body);
    match properties::get(&props, ALIASES_KEY) {
        Some(Value::List(items)) => items
            .iter()
            .map(|item| item.text().trim().to_owned())
            .filter(|text| !text.is_empty())
            .collect(),
        Some(Value::Text(text)) => text
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// A name as entities are matched: [`normalize_title`] with a leading `@`
/// taken off.
pub fn normalize_name(name: &str) -> String {
    normalize_title(name.trim().trim_start_matches('@'))
}

/// The names an entity answers to, normalized and without repeats: its
/// title, then its aliases.
pub fn names_of(title: &str, body: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    std::iter::once(title.to_owned())
        .chain(aliases_of(body))
        .map(|name| normalize_name(&name))
        .filter(|name| !name.is_empty() && seen.insert(name.clone()))
        .collect()
}

/// The aliases as the person wrote them, with the title left out.
pub fn written_aliases(title: &str, body: &str) -> Vec<String> {
    let title = normalize_name(title);
    aliases_of(body)
        .into_iter()
        .filter(|alias| normalize_name(alias) != title)
        .collect()
}

/// A mention found in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    /// 0-based line.
    pub line: u32,
    pub how: How,
    /// The normalized name that matched.
    pub name: String,
}

/// The `@name` mentions in a body, as `(0-based line, name)` with the `@`
/// off and the case kept: an `@` at a line start or after whitespace or `(`,
/// then a letter, then letters, digits, `-`, `_` or `.`. Not inside code or
/// a link, never an e-mail address or a date.
pub fn at_mentions(body: &str) -> Vec<(u32, String)> {
    let code = code_ranges(body);
    let mut out = Vec::new();
    let mut offset = 0usize;
    for (line_no, line) in body.split('\n').enumerate() {
        let start = offset;
        offset += line.len() + 1;
        if !line.contains('@') {
            continue;
        }
        let covered = covered_ranges(line);
        for (at, _) in line.match_indices('@') {
            let abs = start + at;
            if code.iter().any(|r| r.start <= abs && abs < r.end)
                || covered.iter().any(|r| r.start <= at && at < r.end)
            {
                continue;
            }
            let before = line[..at].chars().next_back();
            if !before.is_none_or(|c| c.is_whitespace() || c == '(') {
                continue;
            }
            let rest = &line[at + 1..];
            if !rest.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            let end = rest
                .find(|c: char| !(c.is_alphanumeric() || matches!(c, '-' | '_' | '.')))
                .unwrap_or(rest.len());
            let name = rest[..end].trim_end_matches(['.', '-', '_']);
            // `name@host.tld` was already rejected by the `before` rule; a
            // name that is itself `a@b` can't reach here.
            if !name.is_empty() {
                out.push((line_no as u32, name.to_owned()));
            }
        }
    }
    out
}

/// Every mention in `body` of the entities named in `names`, a map from a
/// normalized name to whatever the caller keys the entity by. A name that
/// maps to `None` is ambiguous and is never matched. One mention per line
/// per name, keeping the surest way it was made. Linear in the body for
/// each name that occurs in it at all.
pub fn scan_mentions<K: Clone + Eq + std::hash::Hash>(
    body: &str,
    names: &HashMap<String, Option<K>>,
) -> Vec<(K, Mention)> {
    if body.is_empty() || names.is_empty() {
        return Vec::new();
    }
    let lower = body.to_lowercase();
    let mut best: HashMap<(K, u32), Mention> = HashMap::new();
    let mut put = |key: K, mention: Mention| {
        let slot = best.entry((key, mention.line));
        slot.and_modify(|old| {
            if mention.how < old.how {
                *old = mention.clone();
            }
        })
        .or_insert(mention);
    };
    for link in note_links(body) {
        if let LinkTarget::Title(title) = &link.target
            && let Some(Some(key)) = names.get(&normalize_name(title))
        {
            put(
                key.clone(),
                Mention {
                    line: link.line,
                    how: How::Link,
                    name: normalize_name(title),
                },
            );
        }
    }
    if lower.contains('@') {
        for (line, name) in at_mentions(body) {
            let name = normalize_name(&name);
            if let Some(Some(key)) = names.get(&name) {
                put(
                    key.clone(),
                    Mention {
                        line,
                        how: How::At,
                        name,
                    },
                );
            }
        }
    }
    for (name, key) in names {
        let Some(key) = key else { continue };
        if name.chars().count() < MIN_WORD_NAME || !lower.contains(name.as_str()) {
            continue;
        }
        for line in mention_lines(body, name) {
            put(
                key.clone(),
                Mention {
                    line,
                    how: How::Word,
                    name: name.clone(),
                },
            );
        }
    }
    let mut out: Vec<(K, Mention)> = best.into_iter().map(|((k, _), m)| (k, m)).collect();
    out.sort_by(|a, b| a.1.line.cmp(&b.1.line).then(a.1.name.cmp(&b.1.name)));
    out
}

/// The body of a new page: front matter marking the kind, the other names
/// when there are any, and nothing else for the person to write.
pub fn new_page_body(kind: EntityKind, aliases: &[String]) -> String {
    let mut body = String::from("---\n");
    body.push_str(&format!("{KIND_KEY}: {}\n", kind.as_str()));
    let aliases: Vec<String> = aliases
        .iter()
        .map(|a| a.trim().to_owned())
        .filter(|a| !a.is_empty())
        .collect();
    if !aliases.is_empty() {
        let list = aliases
            .iter()
            .map(|a| quote_alias(a))
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!("{ALIASES_KEY}: [{list}]\n"));
    }
    body.push_str("---\n");
    body
}

fn quote_alias(alias: &str) -> String {
    let plain = alias
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
        && !alias.starts_with(' ')
        && !alias.ends_with(' ');
    if plain {
        alias.to_owned()
    } else {
        format!("\"{}\"", alias.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// `body` with `alias` added to its `aliases` list (or the list made),
/// leaving every other byte alone. Unchanged when the alias is already a
/// name of the page.
pub fn add_alias(title: &str, body: &str, alias: &str) -> String {
    let alias = alias.trim();
    let wanted = normalize_name(alias);
    if wanted.is_empty() || names_of(title, body).contains(&wanted) {
        return body.to_owned();
    }
    let mut list: Vec<Value> = aliases_of(body).into_iter().map(Value::Text).collect();
    list.push(Value::Text(alias.to_owned()));
    properties::set_property(body, ALIASES_KEY, &Value::List(list))
}

/// What the text of a page is, without its front matter: what the person
/// wrote.
pub fn own_text(body: &str) -> &str {
    match properties::split(body) {
        Some(front) => &body[front.body_start..],
        None => body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn reads_kind_and_names() {
        let body = "---\nkanso: person\naliases: [Sam, \"@sam\"]\n---\nSam runs it.\n";
        assert_eq!(kind_of(body), Some(EntityKind::Person));
        assert_eq!(
            names_of("Sam Moyo", body),
            vec!["sam moyo".to_owned(), "sam".to_owned()]
        );
        assert_eq!(kind_of("---\nkanso: other\n---\n"), None);
        assert_eq!(kind_of("plain"), None);
        assert_eq!(own_text(body), "Sam runs it.\n");
    }

    #[test]
    fn at_mentions_follow_the_rules() {
        let body = "@Sam called\nmail sam@host.com\n(see @sam.moyo.)\n`@code` and [x](@no)\n@2026-11-04 due\n```\n@fenced\n```\n";
        let found = at_mentions(body);
        assert_eq!(
            found,
            vec![(0, "Sam".to_owned()), (2, "sam.moyo".to_owned())]
        );
    }

    #[test]
    fn scans_links_at_and_words_once_per_line() {
        let mut names: HashMap<String, Option<u8>> = HashMap::new();
        names.insert("sam moyo".into(), Some(1));
        names.insert("sam".into(), Some(1));
        names.insert("atlas".into(), Some(2));
        names.insert("dup".into(), None);
        let body = "Met [[Sam Moyo]] about Atlas\n@sam said dup\ntalked to Sam\n";
        let found = scan_mentions(body, &names);
        let lines: Vec<(u8, u32, How)> = found.iter().map(|(k, m)| (*k, m.line, m.how)).collect();
        assert!(lines.contains(&(1, 0, How::Link)));
        assert!(lines.contains(&(2, 0, How::Word)));
        assert!(lines.contains(&(1, 1, How::At)));
        assert!(lines.contains(&(1, 2, How::Word)));
        assert!(!found.iter().any(|(_, m)| m.name == "dup"));
        assert_eq!(found.len(), 4);
    }

    #[test]
    fn new_page_and_alias_round_trip() {
        let body = new_page_body(EntityKind::Person, &["Sam".into(), "Mr \"M\"".into()]);
        assert_eq!(kind_of(&body), Some(EntityKind::Person));
        assert_eq!(
            aliases_of(&body),
            vec!["Sam".to_owned(), "Mr \"M\"".to_owned()]
        );
        let more = add_alias("Sam Moyo", &body, "S. Moyo");
        assert_eq!(aliases_of(&more).len(), 3);
        assert_eq!(add_alias("Sam Moyo", &more, "sam"), more);
        assert_eq!(add_alias("Sam Moyo", &more, "Sam Moyo"), more);
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

        /// `@name` never matches dates, e-mail addresses or code.
        #[test]
        fn prop_at_mentions_never_match_dates_emails_or_code(
            user in "[a-z]{1,8}", host in "[a-z]{1,8}", y in 1000u32..9999, m in 1u32..13, d in 1u32..29,
        ) {
            let body = format!(
                "write {user}@{host}.com\n@{y}-{m:02}-{d:02}\n`@{user}`\n```\n@{user}\n```\n[a](x@{user}) [[@{user}]]\n"
            );
            prop_assert!(at_mentions(&body).is_empty(), "{:?}", at_mentions(&body));
        }

        /// A name written as `@name` after a space is always found, once.
        #[test]
        fn prop_at_mentions_find_a_plain_name(name in "[A-Za-z]([A-Za-z0-9_]{0,9}[A-Za-z0-9])?", lead in "[a-z ]{0,10}") {
            let body = format!("{lead} @{name} hi");
            let found = at_mentions(&body);
            prop_assert_eq!(found, vec![(0, name)]);
        }

        /// Scanning never reports a line the body doesn't have, or an
        /// ambiguous name.
        #[test]
        fn prop_scan_stays_inside_the_body(body in "[a-zA-Z@\\[\\] \n]{0,120}") {
            let mut names: HashMap<String, Option<u8>> = HashMap::new();
            names.insert("sam".into(), Some(1));
            names.insert("amb".into(), None);
            let lines = body.split('\n').count() as u32;
            for (_, m) in scan_mentions(&body, &names) {
                prop_assert!(m.line < lines);
                prop_assert!(m.name != "amb");
            }
        }
    }
}
