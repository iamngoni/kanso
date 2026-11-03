//! People & Projects on the desktop (issue #200), the side that is not
//! drawing: reading a page's sections with the person's settings, rewriting
//! them with the model, and the settings' decisions. The pages, their
//! sections, detection and the AI writing are the engine's
//! (`kanso_engine::entities`, `kanso_engine::pages`), which the phones use
//! too.
//!
//! Nothing here makes or merges a page on its own. A locked note, a note in
//! a locked notebook, a trashed note and, for AI, a note tagged
//! `#ai-exclude` are never a source.

use std::sync::Arc;

use kanso_core::now_ms;
use kanso_engine::entities::page::PageHeader;
use kanso_engine::entities::{EntityPage, EntitySettings};
use kanso_engine::pages::{self, OpenOptions, PageView, RefreshPolicy, write::WriteContext};
use kanso_store::Store;

use crate::ai_feature::Maker;
use crate::config::{AiBackend, AppConfig};

/// A page as the panel shows it.
#[derive(Clone, Debug)]
pub struct Loaded {
    pub header: PageHeader,
    pub view: PageView,
    /// Whether Refresh does anything: AI is on and summaries are wanted.
    pub ai_on: bool,
    /// Sections the person's refresh setting wants written now.
    pub due: Vec<String>,
}

/// Whether AI sections are written at all.
pub fn ai_on(config: &AppConfig) -> bool {
    config.people_projects.ai_sections(config.ai.enabled)
}

/// What opening a page needs to know.
pub fn options(config: &AppConfig) -> OpenOptions {
    OpenOptions {
        ai_on: ai_on(config),
        now: now_ms(),
    }
}

/// When AI sections are rewritten without being asked. A model on this
/// computer or an agent refreshes when a page opens; a provider with an API
/// key only on Refresh, unless the person chose.
pub fn policy(config: &AppConfig) -> RefreshPolicy {
    let settings: &EntitySettings = &config.people_projects;
    match config.ai.backend {
        AiBackend::Agent => match settings.refresh.as_deref() {
            Some("press") => RefreshPolicy::OnPress,
            _ => RefreshPolicy::OnOpen,
        },
        AiBackend::Api => settings.policy(config.ai.provider, &config.ai.base_url),
    }
}

/// Opens note `id` as a page, or `None` when it is not one (or is locked:
/// its own text is all it shows). `show_all` names computed sections to
/// show in full.
pub fn load(
    store: &Store,
    id: &str,
    config: &AppConfig,
    show_all: &[&str],
) -> kanso_store::Result<Option<Loaded>> {
    let Some(page) = EntityPage::load(store, id)? else {
        return Ok(None);
    };
    let options = options(config);
    let view = pages::open(store, &page, options, show_all)?;
    let header = page.header(store)?;
    let due = pages::due_for_refresh(&view, policy(config), options);
    Ok(Some(Loaded {
        header,
        view,
        ai_on: options.ai_on,
        due,
    }))
}

/// "Also known as Sam, @sam · 14 notes · last mentioned 28 Oct".
pub fn header_line(header: &PageHeader) -> String {
    let mut line = String::new();
    if !header.aliases.is_empty() {
        line.push_str(&format!(
            "Also known as {} \u{b7} ",
            header.aliases.join(", ")
        ));
    }
    line.push_str(&format!(
        "{} {}",
        header.notes,
        if header.notes == 1 { "note" } else { "notes" }
    ));
    if let Some(last) = &header.last_mentioned {
        line.push_str(&format!(" \u{b7} last mentioned {last}"));
    }
    line
}

/// Rewrites section `key` of page `id` (or every section that is not kept,
/// when `key` is `None`) with the model `maker` makes. The error is the
/// model's own words, for `page_chrome::ai_trouble` to put plainly.
pub fn refresh(
    store: &Arc<Store>,
    id: &str,
    key: Option<&str>,
    model: &str,
    maker: &Maker,
) -> Result<(), String> {
    let page = EntityPage::load(store, id)
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "This note is not a person or project page.".to_owned())?;
    let shared = maker()?;
    let provider = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let context = WriteContext::new(provider.as_ref(), model, now_ms());
    match key {
        Some(key) => pages::refresh_section(store, &page, key, &context).map(|_| ()),
        None => pages::refresh_all(store, &page, &context).map(|_| ()),
    }
    .map_err(|err| err.to_string())
}

/// The suggestion a selected piece of text names, if it does: the text is
/// one short line whose normalized form is a suggestion's name.
pub fn offer_for<'a, K: Clone>(
    selection: &str,
    offers: &'a [(String, String, K)],
) -> Option<&'a (String, String, K)> {
    let text = selection.trim();
    if text.is_empty() || text.len() > 60 || text.contains('\n') {
        return None;
    }
    let wanted = kanso_core::entities::normalize_name(text);
    offers.iter().find(|(key, name, _)| {
        *key == wanted || kanso_core::entities::normalize_name(name) == wanted
    })
}

/// The name an `@mention` or a `[[wiki-link]]` holding byte `caret` of
/// `text` names, so the page offer shows with the caret in it as well as
/// for a selection. None outside either.
pub fn name_at(text: &str, caret: usize) -> Option<String> {
    if caret > text.len() || !text.is_char_boundary(caret) {
        return None;
    }
    let line_start = text[..caret].rfind('\n').map_or(0, |i| i + 1);
    let line_end = text[caret..].find('\n').map_or(text.len(), |i| caret + i);
    let line = &text[line_start..line_end];
    let at = caret - line_start;
    // `[[Name]]` (or `[[Name|label]]`) around the caret.
    let mut from = 0;
    while let Some(open) = line[from..].find("[[").map(|i| from + i) {
        let Some(close) = line[open + 2..].find("]]").map(|i| open + 2 + i) else {
            break;
        };
        if at >= open && at <= close + 2 {
            let inner = &line[open + 2..close];
            let name = inner.split('|').next().unwrap_or("").trim();
            return (!name.is_empty()).then(|| name.to_owned());
        }
        from = close + 2;
    }
    // `@name` around the caret.
    let is_name = |c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '.';
    let start = line[..at]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_name(*c) || *c == '@')
        .last()
        .map_or(at, |(i, _)| i);
    // A caret just before the `@` is in the mention too.
    let after = at + usize::from(line[at..].starts_with('@'));
    let end = line[after..]
        .char_indices()
        .find(|(_, c)| !is_name(*c))
        .map_or(line.len(), |(i, _)| after + i);
    let word = &line[start..end];
    let name = word.strip_prefix('@')?.trim_end_matches('.');
    let before_ok = line[..start]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric());
    (before_ok && !name.is_empty() && !name.contains('@')).then(|| name.to_owned())
}

/// The names an `@mention` can mean, normalized, each with the page it
/// goes to. A name two pages answer to is left out: it means nothing.
pub type MentionMap = std::collections::HashMap<String, String>;

/// Reads the pages' names. Pure over the rows, so it can be tested.
pub fn mention_map_of(pages: &[(String, Vec<String>)]) -> MentionMap {
    let mut seen: std::collections::HashMap<&str, Option<&str>> = std::collections::HashMap::new();
    for (id, names) in pages {
        for name in names {
            seen.entry(name.as_str())
                .and_modify(|taken| {
                    if *taken != Some(id.as_str()) {
                        *taken = None;
                    }
                })
                .or_insert(Some(id.as_str()));
        }
    }
    seen.into_iter()
        .filter_map(|(name, id)| id.map(|id| (name.to_owned(), id.to_owned())))
        .collect()
}

/// The names of the library's person and project pages.
pub fn mention_map(store: &Store) -> MentionMap {
    let pages: Vec<(String, Vec<String>)> = store
        .entities()
        .unwrap_or_default()
        .into_iter()
        .map(|page| (page.id, page.names))
        .collect();
    mention_map_of(&pages)
}

/// The preview's text with each `@name` that names a page written as a link
/// to it. Text without any comes back as the same shared string.
pub fn rewrite_mentions(text: &gpui_kit::SharedString, map: &MentionMap) -> gpui_kit::SharedString {
    if map.is_empty() || !text.contains('@') {
        return text.clone();
    }
    match kanso_core::entities::rewrite_at_mentions(text, &|name| map.get(name).cloned()) {
        std::borrow::Cow::Borrowed(_) => text.clone(),
        std::borrow::Cow::Owned(rewritten) => rewritten.into(),
    }
}

/// Where an `@name` being typed starts in `before` (the text of the line
/// up to the caret) and what has been typed after the `@`. `None` unless the
/// `@` follows whitespace, `(` or the line's start, so an e-mail address
/// never asks, and nothing but name characters follows it.
pub fn at_query(before: &str) -> Option<(usize, &str)> {
    let at = before.rfind('@')?;
    let query = &before[at + 1..];
    if !query
        .chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return None;
    }
    let ahead = before[..at].chars().next_back();
    ahead
        .is_none_or(|c| c.is_whitespace() || c == '(')
        .then_some((at, query))
}

/// One name `@` can finish: the handle typed after the `@` and the page it
/// goes to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionChoice {
    pub handle: String,
    pub title: String,
}

/// The most names `@` offers at once.
pub const MENTION_CHOICES: usize = 8;

/// The pages' names (`(title, normalized names)`) that `query` can finish,
/// as handles: only a name of one word can follow an `@`. Names starting
/// with the query come first, then those containing it.
pub fn mention_choices(pages: &[(String, Vec<String>)], query: &str) -> Vec<MentionChoice> {
    let query = query.to_lowercase();
    let mut seen = std::collections::HashSet::new();
    let mut out: Vec<(bool, MentionChoice)> = Vec::new();
    for (title, names) in pages {
        for name in names {
            if name.contains(char::is_whitespace) || !seen.insert(name.clone()) {
                continue;
            }
            let starts = name.starts_with(&query);
            if starts || name.contains(&query) {
                out.push((
                    starts,
                    MentionChoice {
                        handle: name.clone(),
                        title: title.clone(),
                    },
                ));
            }
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.handle.cmp(&b.1.handle)));
    out.into_iter()
        .map(|(_, choice)| choice)
        .take(MENTION_CHOICES)
        .collect()
}

#[cfg(test)]
mod tests {
    // no-prop-test: a handful of fixed caret positions.
    #[test]
    fn the_name_at_the_caret() {
        use super::name_at;
        let text = "met @Sam today\nsee [[Sam Moyo|him]] later";
        assert_eq!(name_at(text, 6).as_deref(), Some("Sam"));
        assert_eq!(name_at(text, 4).as_deref(), Some("Sam"));
        assert_eq!(name_at(text, 1), None);
        let link = text.find("Moyo").unwrap();
        assert_eq!(name_at(text, link).as_deref(), Some("Sam Moyo"));
        assert_eq!(name_at(text, text.len()), None);
        assert_eq!(name_at("mail a@b.com", 7), None);
    }

    use super::*;
    use proptest::prelude::*;

    // The query is whatever follows the last `@`, and only after whitespace.
    #[test]
    fn at_query_needs_a_word_start() {
        assert_eq!(at_query("ask @sa"), Some((4, "sa")));
        assert_eq!(at_query("@"), Some((0, "")));
        assert_eq!(at_query("(@sam"), Some((1, "sam")));
        assert_eq!(at_query("mail me@host"), None);
        assert_eq!(at_query("@sam moyo"), None);
    }

    fn pages() -> Vec<(String, Vec<String>)> {
        vec![
            (
                "Sam Moyo".to_owned(),
                vec!["sam moyo".to_owned(), "sam".to_owned()],
            ),
            ("Lusaka".to_owned(), vec!["lusaka".to_owned()]),
        ]
    }

    #[test]
    fn a_selected_name_finds_its_offer() {
        let offers = vec![("tendai chirwa".to_owned(), "Tendai Chirwa".to_owned(), 1u8)];
        assert!(offer_for(" Tendai  Chirwa ", &offers).is_some());
        assert!(offer_for("Tendai", &offers).is_none());
        assert!(offer_for("", &offers).is_none());
        assert!(offer_for("Tendai\nChirwa", &offers).is_none());
    }

    #[test]
    fn a_name_two_pages_share_means_nothing() {
        let pages = vec![
            ("a".to_owned(), vec!["sam".to_owned(), "lusaka".to_owned()]),
            ("b".to_owned(), vec!["sam".to_owned()]),
        ];
        let map = mention_map_of(&pages);
        assert_eq!(map.get("lusaka").map(String::as_str), Some("a"));
        assert!(!map.contains_key("sam"));
    }

    #[test]
    fn a_mention_of_a_page_becomes_a_link() {
        let map = mention_map_of(&[("p1".to_owned(), vec!["sam".to_owned()])]);
        let text = gpui_kit::SharedString::from("ask @Sam, not @bob\n");
        assert_eq!(
            rewrite_mentions(&text, &map).as_ref(),
            "ask [@Sam](inkdrop://note/p1), not @bob\n"
        );
    }

    #[test]
    fn mentions_offer_one_word_names_only() {
        let choices = mention_choices(&pages(), "sa");
        // The name that starts with the query comes before one that has it inside.
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].handle, "sam");
        assert_eq!(choices[0].title, "Sam Moyo");
        assert_eq!(mention_choices(&pages(), "").len(), 2);
        assert!(mention_choices(&pages(), "zz").is_empty());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        // Whatever is typed, every choice contains it, has no space and is
        // at most eight; the same input gives the same choices.
        #[test]
        fn prop_choices_match_the_query(query in "[a-z0-9]{0,4}") {
            let a = mention_choices(&pages(), &query);
            prop_assert!(a.len() <= MENTION_CHOICES);
            for choice in &a {
                prop_assert!(choice.handle.contains(&query));
                prop_assert!(!choice.handle.contains(' '));
            }
            prop_assert_eq!(a, mention_choices(&pages(), &query));
        }
    }

    // no-prop-test: one decision per backend over plain settings.
    #[test]
    fn a_key_provider_refreshes_on_press_and_an_agent_on_open() {
        let mut config = AppConfig::default();
        config.ai.backend = AiBackend::Agent;
        assert_eq!(policy(&config), RefreshPolicy::OnOpen);
        config.people_projects.refresh = Some("press".to_owned());
        assert_eq!(policy(&config), RefreshPolicy::OnPress);

        let mut config = AppConfig::default();
        config.ai.backend = AiBackend::Api;
        config.people_projects.refresh = Some("open".to_owned());
        assert_eq!(policy(&config), RefreshPolicy::OnOpen);
    }

    // no-prop-test: AI sections need both the AI switch and the summaries.
    #[test]
    fn ai_sections_need_ai_and_summaries() {
        let mut config = AppConfig::default();
        assert!(!ai_on(&config));
        config.ai.enabled = true;
        assert!(ai_on(&config));
        config.people_projects.write_with_ai = false;
        assert!(!ai_on(&config));
    }
}
