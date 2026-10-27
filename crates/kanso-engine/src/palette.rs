//! The command palette (⌘K), without a window: the catalog of commands, how
//! well what was typed fits each entry, and the sections a search lists.
//!
//! The desktop palette and the phones' both rank here. With nothing typed
//! the sections are the groups in order ([`Group::ALL`]); with something
//! typed every entry is scored by [`item_score`], each section lists the
//! entries that fit, best first, and the sections are ordered by their best
//! entry. Notes come last, from the same search as Open Note, and keep the
//! ones whose title doesn't fit after the ones that do.
//!
//! A command is listed only when the app has a handler for its id
//! ([`Context::handlers`]) and its [`Needs`] hold, so a package that lands
//! later adds its commands by registering a handler and changes nothing
//! here.

use std::cmp::Reverse;

use kanso_core::NoteStatus;

use crate::fuzzy;
use crate::navigate::quick_open;

pub mod catalog;
pub mod model;

pub use catalog::COMMANDS;
pub use model::*;

/// A match in a keyword counts a little less than the same match in the label.
pub const KEYWORD_PENALTY: i64 = 4;

/// The catalog.
pub fn all() -> &'static [Command] {
    COMMANDS
}

/// The command with this ID.
pub fn command(id: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|command| command.id == id)
}

/// How well `query` fits an entry: its label, or a little less, one of its
/// keywords.
pub fn item_score(query: &str, label: &str, keywords: &[&str]) -> Option<i64> {
    let in_keywords = keywords
        .iter()
        .filter_map(|keyword| fuzzy::score(query, keyword))
        .max()
        .map(|score| score - KEYWORD_PENALTY);
    fuzzy::score(query, label).max(in_keywords)
}

/// Whether `needs` holds in `ctx`.
pub fn applies(needs: Needs, ctx: &Context) -> bool {
    let note = ctx.note.as_ref();
    let in_trash = note.is_some_and(|n| n.trashed);
    let locked = note.is_some_and(|n| n.locked);
    match needs {
        Needs::Nothing => true,
        Needs::Note => note.is_some(),
        Needs::LiveNote => note.is_some() && !in_trash,
        Needs::UnlockedNote => note.is_some() && !locked && !in_trash,
        Needs::LockedNote => locked,
        Needs::Settings => ctx.settings_open,
        Needs::SignedIn => ctx.signed_in,
        Needs::SignedOut => !ctx.signed_in && !ctx.session_held,
        Needs::ImageLine => !in_trash && note.is_some() && ctx.image_line,
        Needs::AudioLine => !in_trash && note.is_some() && ctx.audio_line,
        Needs::SharedNote => note.is_some() && ctx.shared_note,
        Needs::AiOn => ctx.ai_on,
        Needs::AiNote => ctx.ai_on && note.is_some() && !locked && !in_trash,
    }
}

/// The commands of `group` that can be listed now: handled, and with their
/// needs met.
pub fn commands_for<'a>(
    group: Group,
    ctx: &'a Context,
) -> impl Iterator<Item = &'static Command> + 'a {
    COMMANDS
        .iter()
        .filter(move |command| command.group == group)
        .filter(|command| ctx.handlers.iter().any(|id| id == command.id))
        .filter(|command| applies(command.needs, ctx))
}

/// The indices of `scores` listed for `query`: all of them in order for an
/// empty query, otherwise the ones that fit, best first; with
/// `keep_unmatched`, the ones that don't fit follow in order. Also the best
/// score among those listed.
pub fn ranked(
    query: &str,
    scores: &[Option<i64>],
    keep_unmatched: bool,
) -> (Vec<usize>, Option<i64>) {
    let mut order: Vec<usize> = if query.is_empty() {
        (0..scores.len()).collect()
    } else {
        fuzzy::rank_scores(scores)
    };
    if keep_unmatched {
        order.extend((0..scores.len()).filter(|&i| scores[i].is_none()));
    }
    let best = order.first().and_then(|&i| scores[i]);
    (order, best)
}

/// Orders `sections` for `query`: as given for an empty one, otherwise by
/// the best entry of each, the first of equally good ones first.
pub fn order_sections(query: &str, sections: &mut [(Section, Option<i64>)]) {
    if !query.is_empty() {
        sections.sort_by_key(|(_, best)| Reverse(*best));
    }
}

/// A section's entries ranked: `entries` with their scores in, the listed
/// ones in order out. `None` when nothing is listed.
pub fn rank_section(
    query: &str,
    label: &str,
    entries: Vec<(Entry, Option<i64>)>,
    keep_unmatched: bool,
) -> Option<(Section, Option<i64>)> {
    let scores: Vec<Option<i64>> = entries.iter().map(|(_, s)| *s).collect();
    let (order, best) = ranked(query, &scores, keep_unmatched);
    let mut entries: Vec<Option<Entry>> = entries.into_iter().map(|(e, _)| Some(e)).collect();
    let entries: Vec<Entry> = order
        .into_iter()
        .filter_map(|i| entries[i].take())
        .collect();
    (!entries.is_empty()).then(|| {
        (
            Section {
                label: label.to_owned(),
                entries,
            },
            best,
        )
    })
}

/// The palette for `query`: the sections of entries, in the order to list
/// them. Commands, then, in "Go to", the places of the library, and with a
/// query the notes of `ctx`.
pub fn search(query: &str, ctx: &Context) -> Vec<Section> {
    let query = query.trim();
    let mut sections: Vec<(Section, Option<i64>)> = Vec::new();
    let mut push = |found: Option<(Section, Option<i64>)>| sections.extend(found);
    for group in [Group::Note, Group::View] {
        push(rank_section(
            query,
            group.label(),
            command_entries(group, query, ctx),
            false,
        ));
    }
    let mut go_to = place_entries(query, ctx);
    go_to.extend(command_entries(Group::GoTo, query, ctx));
    push(rank_section(query, Group::GoTo.label(), go_to, false));
    push(rank_section(
        query,
        Group::App.label(),
        command_entries(Group::App, query, ctx),
        false,
    ));
    if !query.is_empty() {
        push(rank_section(
            query,
            "Notes",
            quick_open::note_entries(
                query,
                &ctx.notes,
                ctx.note.as_ref().map(|n| n.id.as_str()),
                &ctx.book_names,
            ),
            true,
        ));
    }
    order_sections(query, &mut sections);
    sections.into_iter().map(|(section, _)| section).collect()
}

/// The entries for the commands of `group`, each with its score.
fn command_entries(group: Group, query: &str, ctx: &Context) -> Vec<(Entry, Option<i64>)> {
    let pinned = ctx.note.as_ref().is_some_and(|n| n.pinned);
    commands_for(group, ctx)
        .map(|command| {
            let (label, icon) = match command.toggled {
                Some(toggled) if pinned => (toggled.label, toggled.icon),
                _ => (command.label, command.icon),
            };
            let score = item_score(query, label, command.keywords);
            let entry = Entry {
                target: Target::Command {
                    id: command.id.to_owned(),
                    picker: command.picker,
                },
                label: label.to_owned(),
                icon: icon.to_owned(),
                description: Some(command.description.to_owned()),
                hint: None,
                checked: false,
                depth: 0,
                status: None,
            };
            (entry, score)
        })
        .collect()
}

/// The places of the library the note list can show, each with its score:
/// the built-in ones, the notebooks (flattened once something is typed), the
/// statuses and the trash.
pub fn place_entries(query: &str, ctx: &Context) -> Vec<(Entry, Option<i64>)> {
    let sources = &ctx.places;
    let place = |target: Place, label: &str, icon: &str, count: u32, keywords: &[&str]| {
        let score = item_score(query, label, keywords);
        let entry = Entry {
            checked: sources.showing.as_ref() == Some(&target),
            target: Target::Place(target),
            label: label.to_owned(),
            icon: icon.to_owned(),
            description: None,
            hint: Some(count.to_string()),
            depth: 0,
            status: None,
        };
        (entry, score)
    };
    let mut places = vec![
        place(
            Place::AllNotes,
            "All Notes",
            "files",
            sources.all,
            &["everything", "library"],
        ),
        place(Place::Pinned, "Pinned", "pin", sources.pinned, &["starred"]),
        place(
            Place::Recent,
            "Recent",
            "clock",
            sources.recent,
            &["latest", "edited", "week"],
        ),
        place(
            Place::Tasks,
            "Tasks",
            "list-checks",
            sources.overdue,
            &["todo", "to do", "due", "overdue", "checklist"],
        ),
    ];
    for book in &sources.books {
        let (mut entry, score) = place(
            Place::Notebook(book.id.clone()),
            &book.name,
            "notebook",
            book.count,
            &[],
        );
        // Typing flattens the tree into a ranked list.
        entry.depth = if query.is_empty() { book.depth } else { 0 };
        places.push((entry, score));
    }
    for status in NoteStatus::ALL
        .into_iter()
        .filter(|s| *s != NoteStatus::None)
    {
        let count = sources
            .by_status
            .iter()
            .find(|(name, _)| name == status.as_str())
            .map_or(0, |(_, count)| *count);
        let (mut entry, score) = place(
            Place::Status(status.as_str().to_owned()),
            status.label(),
            "circle-dot",
            count,
            &["status"],
        );
        entry.status = Some(status.as_str().to_owned());
        places.push((entry, score));
    }
    if sources.shared > 0 || ctx.signed_in {
        places.push(place(
            Place::Shared,
            "Shared with me",
            "users",
            sources.shared,
            &["shared", "collaboration", "invitations", "others"],
        ));
    }
    places.push(place(
        Place::Trash,
        "Trash",
        "trash",
        sources.trash,
        &["deleted"],
    ));
    places
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use proptest::prelude::*;

    use super::*;

    fn all_handlers() -> Vec<String> {
        COMMANDS.iter().map(|c| c.id.to_owned()).collect()
    }

    /// A context where every command has a handler.
    fn context(note: Option<OpenNote>) -> Context {
        Context {
            note,
            signed_in: true,
            ai_on: true,
            settings_open: true,
            image_line: true,
            audio_line: true,
            shared_note: true,
            handlers: all_handlers(),
            ..Context::default()
        }
    }

    fn open(locked: bool, trashed: bool, pinned: bool) -> Option<OpenNote> {
        Some(OpenNote {
            id: "note:open".into(),
            locked,
            trashed,
            pinned,
        })
    }

    fn arb_context() -> impl Strategy<Value = Context> {
        (
            prop::option::of((any::<bool>(), any::<bool>(), any::<bool>())),
            (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()),
            (any::<bool>(), any::<bool>(), any::<bool>()),
            prop::sample::subsequence(all_handlers(), 0..=COMMANDS.len()),
        )
            .prop_map(
                |(note, (signed_in, held, ai_on, settings), (image, audio, shared), handlers)| {
                    Context {
                        note: note.map(|(locked, trashed, pinned)| OpenNote {
                            id: "note:open".into(),
                            locked,
                            trashed,
                            pinned,
                        }),
                        signed_in,
                        session_held: held,
                        ai_on,
                        settings_open: settings,
                        image_line: image,
                        audio_line: audio,
                        shared_note: shared,
                        handlers,
                        ..Context::default()
                    }
                },
            )
    }

    fn command_ids(sections: &[Section]) -> Vec<String> {
        sections
            .iter()
            .flat_map(|s| &s.entries)
            .filter_map(|e| match &e.target {
                Target::Command { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect()
    }

    // no-prop-test: the catalog is a fixed table; these are its invariants.
    #[test]
    fn the_catalog_is_well_formed() {
        let mut seen = HashSet::new();
        for command in COMMANDS {
            assert!(seen.insert(command.id), "{} is listed twice", command.id);
            assert!(!command.label.is_empty() && !command.description.is_empty());
            let kebab = |icon: &str| {
                !icon.is_empty()
                    && icon
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            };
            assert!(kebab(command.icon), "{}: {}", command.id, command.icon);
            if let Some(toggled) = command.toggled {
                assert!(kebab(toggled.icon));
            }
        }
        // The commands that open a list.
        let pickers: Vec<_> = COMMANDS
            .iter()
            .filter(|c| c.picker.is_some())
            .map(|c| (c.id, c.picker.unwrap()))
            .collect();
        assert_eq!(
            pickers,
            [
                ("new-from-template", Picker::Template),
                ("move", Picker::Notebook),
                ("add-tag", Picker::Tag),
                ("status", Picker::Status),
                ("resize-image", Picker::ImageSize),
                ("word-goal", Picker::WordGoal),
                ("go-to-notebook", Picker::GoToNotebook),
                ("go-to-tag", Picker::GoToTag),
            ]
        );
    }

    // no-prop-test: three fixed states of one command.
    #[test]
    fn pinning_toggles_its_label() {
        let find = |note| {
            let sections = search("pin", &context(note));
            sections
                .iter()
                .flat_map(|s| &s.entries)
                .find(|e| matches!(&e.target, Target::Command { id, .. } if id == "pin"))
                .map(|e| (e.label.clone(), e.icon.clone()))
        };
        assert_eq!(
            find(open(false, false, false)),
            Some(("Pin note".into(), "pin".into()))
        );
        assert_eq!(
            find(open(false, false, true)),
            Some(("Unpin note".into(), "pin-off".into()))
        );
        // Not in the trash.
        assert_eq!(find(open(false, true, false)), None);
    }

    proptest! {
        /// Every command listed has a handler and its needs hold, whatever
        /// the context and the query.
        #[test]
        fn prop_listed_commands_are_handled_and_apply(
            ctx in arb_context(),
            query in "[a-z ]{0,6}",
        ) {
            let sections = search(&query, &ctx);
            for id in command_ids(&sections) {
                prop_assert!(ctx.handlers.contains(&id), "{id} has no handler");
                let command = command(&id).unwrap();
                prop_assert!(applies(command.needs, &ctx), "{id} doesn't apply");
            }
        }

        /// A command with a handler whose needs hold is listed when nothing
        /// is typed, and a command without one never is.
        #[test]
        fn prop_empty_query_lists_exactly_the_applicable_handled_commands(ctx in arb_context()) {
            let listed: HashSet<String> = command_ids(&search("", &ctx)).into_iter().collect();
            let expected: HashSet<String> = COMMANDS
                .iter()
                .filter(|c| ctx.handlers.iter().any(|h| h == c.id) && applies(c.needs, &ctx))
                .map(|c| c.id.to_owned())
                .collect();
            prop_assert_eq!(listed, expected);
        }

        /// Typing a command's label puts that command first among the
        /// commands.
        #[test]
        fn prop_a_label_ranks_its_command_first(index in 0..COMMANDS.len()) {
            let command = &COMMANDS[index];
            let ctx = context(open(false, false, false));
            prop_assume!(applies(command.needs, &ctx));
            let ids = command_ids(&search(command.label, &ctx));
            prop_assert_eq!(ids.first().map(String::as_str), Some(command.id), "{:?}", ids);
        }

        /// Nothing typed lists the groups in order, each command in the
        /// catalog's order, with nothing from the notes.
        #[test]
        fn prop_empty_query_keeps_the_catalog_order(ctx in arb_context()) {
            let sections = search("", &ctx);
            let order: Vec<&str> = sections.iter().map(|s| s.label.as_str()).collect();
            let wanted: Vec<&str> = Group::ALL.iter().map(|g| g.label()).collect();
            let mut at = 0;
            for label in &order {
                let found = wanted[at..].iter().position(|w| w == label);
                prop_assert!(found.is_some(), "{order:?} is out of order");
                at += found.unwrap() + 1;
            }
            prop_assert!(!order.contains(&"Notes"));
            // Within a group the commands keep the catalog's order.
            let ids = command_ids(&sections);
            for group in Group::ALL {
                let in_group: Vec<usize> = ids
                    .iter()
                    .filter_map(|id| COMMANDS.iter().position(|c| c.id == id && c.group == group))
                    .collect();
                prop_assert!(in_group.windows(2).all(|w| w[0] < w[1]));
            }
        }

        /// The ranking only ever lists what fits (or a note found by its
        /// text), best first within a section.
        #[test]
        fn prop_sections_are_ranked_best_first(query in "[a-z]{1,4}") {
            let ctx = context(open(false, false, false));
            for section in search(&query, &ctx) {
                // Notes carry Open Note's own score, which also matches the
                // text, so a listed note need not have the query in its
                // title. The claims here are about the commands.
                let scores: Vec<Option<i64>> = section
                    .entries
                    .iter()
                    .filter(|e| matches!(e.target, Target::Command { .. }))
                    .map(|e| match &e.target {
                        Target::Command { id, .. } => {
                            let keywords = command(id).map_or(&[][..], |c| c.keywords);
                            item_score(&query, &e.label, keywords)
                        }
                        _ => None,
                    })
                    .collect();
                prop_assert!(
                    scores.iter().all(Option::is_some),
                    "{} has an unranked command: {:?}",
                    section.label,
                    scores
                );
                let fitting: Vec<i64> = scores.iter().flatten().copied().collect();
                prop_assert!(fitting.windows(2).all(|w| w[0] >= w[1]), "{:?}", scores);
            }
        }
    }

    // no-prop-test: the places of one fixed library, named exactly.
    #[test]
    fn the_places_are_listed_with_their_counts() {
        let mut ctx = context(None);
        ctx.places = PlaceSources {
            all: 5,
            trash: 1,
            books: vec![
                BookSource {
                    id: "book:a".into(),
                    name: "Work".into(),
                    depth: 0,
                    count: 3,
                },
                BookSource {
                    id: "book:b".into(),
                    name: "Plans".into(),
                    depth: 1,
                    count: 2,
                },
            ],
            showing: Some(Place::Notebook("book:b".into())),
            ..PlaceSources::default()
        };
        let go_to = search("", &ctx)
            .into_iter()
            .find(|s| s.label == "Go to")
            .unwrap();
        let labels: Vec<&str> = go_to.entries.iter().map(|e| e.label.as_str()).collect();
        assert_eq!(
            &labels[..9],
            [
                "All Notes",
                "Pinned",
                "Recent",
                "Tasks",
                "Work",
                "Plans",
                "Active",
                "On hold",
                "Completed"
            ]
        );
        let plans = go_to.entries.iter().find(|e| e.label == "Plans").unwrap();
        assert_eq!(
            (plans.depth, plans.checked, plans.hint.as_deref()),
            (1, true, Some("2"))
        );
        // Typing flattens the tree.
        let typed = search("plans", &ctx);
        let plans = typed
            .iter()
            .flat_map(|s| &s.entries)
            .find(|e| e.label == "Plans")
            .unwrap();
        assert_eq!(plans.depth, 0);
        // Shared is offered when signed in.
        assert!(labels.contains(&"Shared with me") && labels.contains(&"Trash"));
    }
}
