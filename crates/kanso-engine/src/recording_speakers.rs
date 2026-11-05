//! Who said what in a recording's transcript, and renaming them (issue
//! #38, step 13).
//!
//! The names are text in the note: `[00:22](kanso-audio://file:abc?t=22)
//! Speaker 2: The beta is on track.` There is no hidden table; renaming
//! "Speaker 2" to "Priya" rewrites that name at the start of each of its
//! paragraphs, inside the transcript's markers and nowhere else, so prose
//! the person wrote around or under it is never touched. Giving two voices
//! the same name merges them. Pure.

use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::batch::Edit;
use crate::recording_note::{TRANSCRIPT_CLOSE, TRANSCRIPT_OPEN, parse_moment, transcript_range};
pub use crate::recording_speakers_model::{
    MAX_NAME_CHARS, SpeakerName, SpeakerOffer, SpeakersError,
};

/// The names Kanso writes itself, which are always taken for names.
fn is_ours(name: &str) -> bool {
    name == "You"
        || name == "Meeting"
        || name
            .strip_prefix("Speaker ")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Whether `name` could be a name before a paragraph: short, a few words,
/// letters, numbers and ordinary name punctuation.
fn looks_like_a_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().count() <= MAX_NAME_CHARS
        && name == name.trim()
        && name.split(' ').count() <= 4
        && name.split(' ').all(|w| !w.is_empty())
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '\'' | '.' | '’'))
}

/// A name as it will be written: whitespace tidied, anything that could
/// break the line or the markers taken out, at most [`MAX_NAME_CHARS`]
/// characters. `None` when nothing is left. Pure.
pub fn clean_name(raw: &str) -> Option<String> {
    let kept: String = raw
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '\'' | '.' | '’') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let name: String = kept
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_NAME_CHARS)
        .collect();
    let name = name.trim().to_owned();
    (!name.is_empty()).then_some(name)
}

/// Where in `line` the text of a paragraph starts, after its moment link
/// and the space that follows; `None` for a line that is not a paragraph of
/// a recording.
fn after_link(line: &str) -> Option<usize> {
    if !line.starts_with('[') {
        return None;
    }
    let open = line.find("](")? + 2;
    let close = line[open..].find(')')? + open;
    parse_moment(&line[open..close])?;
    let rest = &line[close + 1..];
    Some(close + 1 + (rest.len() - rest.trim_start().len()))
}

/// The name a paragraph line begins with, as `(start, end)` bytes in the
/// line, when it has one: some words, then a colon and a space.
fn name_range(line: &str) -> Option<(usize, usize)> {
    let at = after_link(line)?;
    let text = &line[at..];
    let colon = text.find(": ")?;
    let name = &text[..colon];
    looks_like_a_name(name).then_some((at, at + colon))
}

/// The transcript block the note's first recording has, as the file ID it
/// belongs to. `None` when the note has none. Pure.
pub fn first_recording(body: &str) -> Option<String> {
    let mut from = 0;
    while let Some(open) = body[from..].find(TRANSCRIPT_OPEN).map(|at| at + from) {
        let inner_start = open + TRANSCRIPT_OPEN.len();
        let close = body[inner_start..].find(TRANSCRIPT_CLOSE)?;
        let inner = &body[inner_start..inner_start + close];
        let id = inner
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .and_then(|line| {
                let start = line.find("](")? + 2;
                let end = line[start..].find(')')? + start;
                parse_moment(&line[start..end]).map(|m| m.file_id)
            });
        if let Some(id) = id
            && transcript_range(body, &id).is_some_and(|r| r.start == open)
        {
            return Some(id);
        }
        from = inner_start + close + TRANSCRIPT_CLOSE.len();
    }
    None
}

/// The names that begin paragraphs of `file_id`'s transcript, in the order
/// they first appear. A name counts when Kanso wrote it (`You`, `Meeting`,
/// `Speaker 3`), when it begins at least two paragraphs, or when most of
/// the transcript's paragraphs begin with a name (so a renamed speaker who
/// said one thing still shows), but not otherwise: a colon in someone's
/// sentence is not a speaker. Empty when the note has no marked transcript
/// for `file_id`. Pure.
pub fn speakers(body: &str, file_id: &str) -> Vec<SpeakerName> {
    let Some(range) = transcript_range(body, file_id) else {
        return Vec::new();
    };
    let mut seen: Vec<SpeakerName> = Vec::new();
    let mut paragraphs = 0usize;
    for line in body[range].lines() {
        if after_link(line).is_some() {
            paragraphs += 1;
        }
        let Some((from, to)) = name_range(line) else {
            continue;
        };
        let name = &line[from..to];
        match seen.iter_mut().find(|s| s.name == name) {
            Some(s) => s.paragraphs += 1,
            None => seen.push(SpeakerName {
                name: name.to_owned(),
                paragraphs: 1,
            }),
        }
    }
    let named: usize = seen.iter().map(|s| s.paragraphs as usize).sum();
    let mostly_named = named * 2 > paragraphs;
    seen.retain(|s| mostly_named || is_ours(&s.name) || s.paragraphs >= 2);
    seen
}

/// `body` with every paragraph of `file_id`'s transcript that begins with
/// `from` beginning with `to` (cleaned by [`clean_name`]) instead. `None`
/// when nothing would change: no such transcript, no such name, or the new
/// name is the old one. Nothing outside the transcript's markers moves.
/// Pure.
pub fn rename(body: &str, file_id: &str, from: &str, to: &str) -> Option<String> {
    rename_many(body, file_id, &[(from.to_owned(), to.to_owned())])
}

/// [`rename`] for several names at once: every paragraph is looked at once,
/// by the name it has now, so swapping two names, or naming one after
/// another, does not cascade. `None` when no paragraph changes. Pure.
pub fn rename_many(body: &str, file_id: &str, changes: &[(String, String)]) -> Option<String> {
    let range = transcript_range(body, file_id)?;
    let wanted: Vec<(&str, String)> = changes
        .iter()
        .filter_map(|(from, to)| Some((from.as_str(), clean_name(to)?)))
        .filter(|(from, to)| from != to)
        .collect();
    let mut changed = false;
    let mut block = String::with_capacity(range.len() + 16);
    for (i, line) in body[range.clone()].split('\n').enumerate() {
        if i > 0 {
            block.push('\n');
        }
        let new = name_range(line).and_then(|(a, b)| {
            wanted
                .iter()
                .find(|(from, _)| *from == &line[a..b])
                .map(|(_, to)| (a, b, to))
        });
        match new {
            Some((a, b, to)) => {
                changed = true;
                block.push_str(&line[..a]);
                block.push_str(to);
                block.push_str(&line[b..]);
            }
            None => block.push_str(line),
        }
    }
    changed.then(|| format!("{}{block}{}", &body[..range.start], &body[range.end..]))
}

/// The seconds a paragraph line's moment link points at, when the line is
/// a paragraph of a recording.
fn moment_seconds(line: &str) -> Option<u32> {
    after_link(line)?;
    let open = line.find("](")? + 2;
    let close = line[open..].find(')')? + open;
    parse_moment(&line[open..close]).map(|m| m.seconds)
}

/// One of Kanso's labels in a new transcript, and what the old transcript
/// said at its paragraphs' moments.
struct Tally<'a> {
    label: &'a str,
    /// Its paragraphs that fall in an old paragraph.
    heard: usize,
    /// The given names those old paragraphs had, with how many each.
    votes: Vec<(&'a str, usize)>,
}

/// `new_body`'s transcript of `file_id`, from a fresh transcription of the
/// same recording, with the names the person gave its voices in
/// `old_body`'s transcript kept: a voice Kanso labels (`Speaker 2`, `You`)
/// takes the name most of its paragraphs had in the old transcript at the
/// same moments, when more than half of them had that one name and it is
/// not one of Kanso's own labels. So transcribing a recording again does
/// not undo "Rename Speakers". `None` when no name carries over. Pure.
pub fn carry_names(old_body: &str, new_body: &str, file_id: &str) -> Option<String> {
    let old_range = transcript_range(old_body, file_id)?;
    let given: Vec<String> = speakers(old_body, file_id)
        .into_iter()
        .map(|s| s.name)
        .filter(|name| !is_ours(name))
        .collect();
    if given.is_empty() {
        return None;
    }
    // The old paragraphs in order: when each starts, and its given name.
    let old: Vec<(u32, Option<&str>)> = old_body[old_range.clone()]
        .lines()
        .filter_map(|line| {
            let at = moment_seconds(line)?;
            let name = name_range(line)
                .map(|(a, b)| &line[a..b])
                .filter(|name| given.iter().any(|g| g == name));
            Some((at, name))
        })
        .collect();
    let new_range = transcript_range(new_body, file_id)?;
    // Per label of ours in the new transcript: how many of its paragraphs
    // fall in an old paragraph, and the votes for each old name.
    let mut tallies: Vec<Tally> = Vec::new();
    for line in new_body[new_range].lines() {
        let (Some(at), Some((a, b))) = (moment_seconds(line), name_range(line)) else {
            continue;
        };
        let label = &line[a..b];
        if !is_ours(label) {
            continue;
        }
        let Some(&(_, name)) = old.iter().rev().find(|(start, _)| *start <= at) else {
            continue;
        };
        let at = match tallies.iter().position(|t| t.label == label) {
            Some(at) => at,
            None => {
                tallies.push(Tally {
                    label,
                    heard: 0,
                    votes: Vec::new(),
                });
                tallies.len() - 1
            }
        };
        let tally = &mut tallies[at];
        tally.heard += 1;
        if let Some(name) = name {
            match tally.votes.iter_mut().find(|(n, _)| *n == name) {
                Some((_, count)) => *count += 1,
                None => tally.votes.push((name, 1)),
            }
        }
    }
    let changes: Vec<(String, String)> = tallies
        .into_iter()
        .filter_map(
            |Tally {
                 label,
                 heard,
                 votes,
             }| {
                // The first name to reach the most votes, for a stable answer.
                let (name, count) = votes.into_iter().fold(
                    None,
                    |best: Option<(&str, usize)>, (n, c)| match best {
                        Some((_, b)) if b >= c => best,
                        _ => Some((n, c)),
                    },
                )?;
                (count * 2 > heard).then(|| (label.to_owned(), name.to_owned()))
            },
        )
        .collect();
    rename_many(new_body, file_id, &changes)
}

/// The changes a form asks for: each of `names` paired with what was typed
/// for it, left out when the typed name is empty after cleaning or is the
/// name it already has. Pure.
pub fn changes(names: &[SpeakerName], typed: &[String]) -> Vec<(String, String)> {
    names
        .iter()
        .zip(typed)
        .filter_map(|(speaker, typed)| {
            let to = clean_name(typed)?;
            (to != speaker.name).then(|| (speaker.name.clone(), to))
        })
        .collect()
}

/// Why note `id` may not have its speakers renamed, if it may not: it is
/// gone, locked (its words are sealed), in the trash, or tagged
/// `#ai-exclude`. Fails closed: a check that cannot be made refuses.
pub fn refusal(store: &Store, id: &str) -> Option<SpeakersError> {
    let note = match store.note(id) {
        Ok(Some(note)) => note,
        Ok(None) => return Some(SpeakersError::Missing),
        Err(err) => return Some(err.into()),
    };
    if store.note_is_locked(id).unwrap_or(true) {
        return Some(SpeakersError::Locked);
    }
    if note.book_id == TRASH_BOOK_ID || !crate::assist::eligible(store, id) {
        return Some(SpeakersError::KeptOut);
    }
    None
}

/// The speakers of note `id`'s first recording, read from the store. Not
/// for a locked note, whose words are sealed, nor one in the trash or
/// tagged `#ai-exclude` ([`refusal`]).
pub fn offer(store: &Store, id: &str) -> Result<SpeakerOffer, SpeakersError> {
    if let Some(why) = refusal(store, id) {
        return Err(why);
    }
    let note = store.note(id)?.ok_or(SpeakersError::Missing)?;
    let file_id = first_recording(&note.body).ok_or(SpeakersError::NoTranscript)?;
    let names = speakers(&note.body, &file_id);
    if names.is_empty() {
        return Err(SpeakersError::NoSpeakers);
    }
    Ok(SpeakerOffer { file_id, names })
}

/// Applies `changes` to the first recording's transcript in note `id`, in
/// one `put`, so Undo and the note's history bring the old names back. The
/// note is read again first and a note edited meanwhile is refused
/// ([`SpeakersError::Changed`]), not overwritten.
pub fn apply_renames(
    store: &Store,
    id: &str,
    changes: &[(String, String)],
    origin: Origin,
) -> Result<Edit, SpeakersError> {
    let offered = offer(store, id)?;
    let note = store.note(id)?.ok_or(SpeakersError::Missing)?;
    let body =
        rename_many(&note.body, &offered.file_id, changes).ok_or(SpeakersError::NothingToChange)?;
    let result = store.put(
        DocType::Note,
        json!({"_id": id, "_rev": note.rev, "body": body}),
        Put::Merge,
        origin,
    );
    match result {
        Ok(_) => Ok(Edit {
            id: id.to_owned(),
            change: json!({"_id": id, "body": body}),
            undo: json!({"_id": id, "body": note.body}),
        }),
        Err(StoreError::Conflict) => Err(SpeakersError::Changed),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording_note::{Segment, render_transcript};
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn block(file: &str, lines: &[(u32, Option<&str>, &str)]) -> String {
        let segments: Vec<Segment> = lines
            .iter()
            .map(|(start_secs, who, text)| Segment {
                start_secs: *start_secs,
                text: (*text).into(),
                speaker: who.map(str::to_owned),
            })
            .collect();
        render_transcript(file, &segments).unwrap()
    }

    fn meeting() -> String {
        let b = block(
            "file:abc",
            &[
                (0, Some("You"), "Morning."),
                (20, Some("Speaker 1"), "Beta ships on the 6th."),
                (40, Some("Speaker 2"), "Onboarding is the risk."),
                (60, Some("Speaker 1"), "Agreed: ship anyway."),
            ],
        );
        format!("# Meeting\n\nSpeaker 1: my own note\n\n{b}\n\nSpeaker 1 owes a draft.")
    }

    #[test]
    fn a_new_transcript_keeps_the_names_given_to_its_voices() {
        let named = rename_many(
            &meeting(),
            "file:abc",
            &[
                ("Speaker 1".into(), "Priya".into()),
                ("You".into(), "Ngoni".into()),
            ],
        )
        .unwrap();
        // Transcribed again, the voices came out numbered the other way
        // round: the names follow the voices, not the numbers.
        let fresh = block(
            "file:abc",
            &[
                (0, Some("You"), "Morning!"),
                (20, Some("Speaker 2"), "Beta ships on the 6th."),
                (40, Some("Speaker 1"), "Onboarding is the risk."),
                (60, Some("Speaker 2"), "Agreed: ship anyway."),
            ],
        );
        let kept = carry_names(&named, &fresh, "file:abc").unwrap();
        let shown: Vec<_> = speakers(&kept, "file:abc")
            .into_iter()
            .map(|s| (s.name, s.paragraphs))
            .collect();
        assert_eq!(
            shown,
            vec![
                ("Ngoni".to_owned(), 1),
                ("Priya".to_owned(), 2),
                ("Speaker 1".to_owned(), 1)
            ]
        );
        assert!(kept.contains("?t=0) Ngoni: Morning!"), "{kept}");
        // Nothing renamed: nothing to keep.
        assert_eq!(carry_names(&meeting(), &fresh, "file:abc"), None);
        // No old transcript, or a fresh one without labels: nothing either.
        assert_eq!(carry_names("no transcript", &fresh, "file:abc"), None);
        let plain = block("file:abc", &[(0, None, "Morning."), (20, None, "Beta.")]);
        assert_eq!(carry_names(&named, &plain, "file:abc"), None);
    }

    #[test]
    fn speakers_are_listed_in_order_with_their_paragraphs() {
        let found = speakers(&meeting(), "file:abc");
        let shown: Vec<_> = found
            .iter()
            .map(|s| (s.name.as_str(), s.paragraphs))
            .collect();
        assert_eq!(shown, vec![("You", 1), ("Speaker 1", 2), ("Speaker 2", 1)]);
        assert_eq!(first_recording(&meeting()).as_deref(), Some("file:abc"));
        assert_eq!(first_recording("no transcript"), None);
        assert!(speakers("no transcript", "file:abc").is_empty());
    }

    #[test]
    fn renaming_changes_only_the_names_inside_the_markers() {
        let body = meeting();
        let out = rename(&body, "file:abc", "Speaker 1", "Priya").unwrap();
        assert!(out.contains("?t=20) Priya: Beta ships on the 6th."));
        assert!(out.contains("?t=60) Priya: Agreed: ship anyway."));
        assert!(out.contains("?t=40) Speaker 2: Onboarding"));
        // The person's own lines around the block are untouched.
        assert!(out.starts_with("# Meeting\n\nSpeaker 1: my own note\n\n"));
        assert!(out.ends_with("\n\nSpeaker 1 owes a draft."));
        // The renamed block is still the machine's, so it can be renamed again
        // and re-transcribed.
        assert_eq!(first_recording(&out).as_deref(), Some("file:abc"));
        let again = rename(&out, "file:abc", "Priya", "Priya Patel").unwrap();
        assert!(again.contains("Priya Patel: Beta"));
    }

    #[test]
    fn two_names_made_one_merge_the_voices() {
        let out = rename(&meeting(), "file:abc", "Speaker 2", "Speaker 1").unwrap();
        let found = speakers(&out, "file:abc");
        assert_eq!(found.len(), 2);
        assert_eq!(found[1].paragraphs, 3);
    }

    #[test]
    fn a_colon_in_a_sentence_is_not_a_speaker() {
        let b = block(
            "file:abc",
            &[
                (0, None, "Agenda: the roadmap."),
                (20, None, "Decision: ship."),
                (40, Some("Speaker 1"), "Fine."),
                (60, None, "More talk."),
                (80, None, "Even more."),
                (100, None, "Last."),
            ],
        );
        let found = speakers(&b, "file:abc");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Speaker 1");
        // Only the sheet decides what to offer; asking for a name that is
        // not at the start of a paragraph changes nothing.
        assert_eq!(rename(&b, "file:abc", "the roadmap", "Priya"), None);
    }

    #[test]
    fn nothing_to_change_is_none() {
        let body = meeting();
        assert_eq!(rename(&body, "file:abc", "Speaker 1", "Speaker 1"), None);
        assert_eq!(rename(&body, "file:abc", "Nobody", "Priya"), None);
        assert_eq!(rename(&body, "file:abc", "Speaker 1", "  :<>  "), None);
        assert_eq!(rename(&body, "file:other", "Speaker 1", "Priya"), None);
        assert_eq!(clean_name("  Priya\n[x](y) "), Some("Priya x y".into()));
        let marker = clean_name("<!-- kanso:transcript -->").unwrap();
        assert!(!marker.contains(['<', '>', ':']) && !marker.contains("<!--"));
    }

    #[test]
    fn names_can_be_swapped_without_cascading() {
        let body = meeting();
        let out = rename_many(
            &body,
            "file:abc",
            &[
                ("Speaker 1".into(), "Speaker 2".into()),
                ("Speaker 2".into(), "Speaker 1".into()),
            ],
        )
        .unwrap();
        assert!(out.contains("?t=20) Speaker 2: Beta"));
        assert!(out.contains("?t=40) Speaker 1: Onboarding"));
        // Naming one after another merges them, once.
        let out = rename_many(
            &body,
            "file:abc",
            &[
                ("Speaker 1".into(), "Speaker 2".into()),
                ("Speaker 2".into(), "Priya".into()),
            ],
        )
        .unwrap();
        assert!(out.contains("?t=20) Speaker 2: Beta"));
        assert!(out.contains("?t=40) Priya: Onboarding"));
    }

    #[test]
    fn the_form_asks_only_for_what_changed() {
        let names = speakers(&meeting(), "file:abc");
        let typed = ["You".to_owned(), "  Priya ".to_owned(), String::new()];
        assert_eq!(
            changes(&names, &typed),
            vec![("Speaker 1".to_owned(), "Priya".to_owned())]
        );
    }

    fn library(body: &str) -> (tempfile::TempDir, Store, String) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        let book = store
            .put(
                DocType::Book,
                json!({"name": "Work"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let id = store
            .put(
                DocType::Note,
                json!({"title": "Meeting", "body": body, "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        (dir, store, id)
    }

    #[test]
    fn the_store_offers_names_and_renames_in_one_undoable_write() {
        let (_dir, store, id) = library(&meeting());
        let offered = offer(&store, &id).unwrap();
        assert_eq!(offered.file_id, "file:abc");
        assert_eq!(offered.names.len(), 3);
        let edit = apply_renames(
            &store,
            &id,
            &[("Speaker 1".into(), "Priya".into())],
            Origin::Ui,
        )
        .unwrap();
        let now = store.note(&id).unwrap().unwrap();
        assert!(now.body.contains("Priya: Beta ships"));
        // The undo restores the exact old body.
        assert_eq!(edit.undo["body"], json!(meeting()));
        assert_eq!(edit.change["body"], json!(now.body));
        // Nothing to change is refused, and so is a note with no transcript.
        assert!(matches!(
            apply_renames(
                &store,
                &id,
                &[("Speaker 1".into(), "Speaker 1".into())],
                Origin::Ui
            ),
            Err(SpeakersError::NothingToChange)
        ));
        let (_dir, store, id) = library("just prose");
        assert!(matches!(
            offer(&store, &id),
            Err(SpeakersError::NoTranscript)
        ));
        assert!(matches!(
            offer(&store, "note:nope"),
            Err(SpeakersError::Missing)
        ));
    }

    #[test]
    fn locked_trashed_and_ai_exclude_notes_are_refused_and_left_unchanged() {
        let (_dir, store, plain) = library(&meeting());
        let book = store.note(&plain).unwrap().unwrap().book_id;
        let put = |extra: serde_json::Value| {
            let mut fields = json!({"title": "Meeting", "body": meeting(), "bookId": book});
            fields
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            store
                .put(DocType::Note, fields, Put::Upsert, Origin::Ui)
                .unwrap()
                .id
        };
        let locked = put(json!({}));
        store
            .put(
                DocType::Note,
                json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
                Put::Merge,
                Origin::Sync,
            )
            .unwrap();
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let excluded = put(json!({"tags": [tag]}));
        let trashed = put(json!({"bookId": TRASH_BOOK_ID}));
        let rename = [("Speaker 1".to_owned(), "Priya".to_owned())];
        assert!(refusal(&store, &plain).is_none());
        assert!(matches!(offer(&store, &locked), Err(SpeakersError::Locked)));
        assert!(matches!(
            apply_renames(&store, &locked, &rename, Origin::Ui),
            Err(SpeakersError::Locked)
        ));
        for id in [&excluded, &trashed] {
            assert!(
                matches!(offer(&store, id), Err(SpeakersError::KeptOut)),
                "{id}"
            );
            assert!(matches!(
                apply_renames(&store, id, &rename, Origin::Ui),
                Err(SpeakersError::KeptOut)
            ));
        }
        for id in [&locked, &excluded, &trashed] {
            let body = store.note(id).unwrap().unwrap().body;
            assert!(!body.contains("Priya"), "{id} was changed");
        }
        assert!(matches!(
            refusal(&store, "note:missing"),
            Some(SpeakersError::Missing)
        ));
    }

    /// A transcript of `speakers` taking turns.
    fn turns(names: &[String]) -> String {
        let lines: Vec<(u32, Option<&str>, &str)> = names
            .iter()
            .enumerate()
            .map(|(i, n)| (i as u32 * 20, Some(n.as_str()), "Words here."))
            .collect();
        block("file:p", &lines)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 96,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// Renaming rewrites exactly the paragraphs that had the name and
        /// keeps the prose outside the markers byte for byte.
        #[test]
        fn prop_rename_touches_only_that_speaker(
            who in proptest::collection::vec(0usize..4, 1..12),
            target in 0usize..4,
            new in "[A-Z][a-z]{2,8}",
            before in "[a-z \n]{0,20}",
            after in "[a-z \n]{0,20}",
        ) {
            let names: Vec<String> = who.iter().map(|w| format!("Speaker {}", w + 1)).collect();
            let body = format!("{before}{}{after}", turns(&names));
            let from = format!("Speaker {}", target + 1);
            match rename(&body, "file:p", &from, &new) {
                None => prop_assert!(!names.contains(&from)),
                Some(out) => {
                    prop_assert!(out.starts_with(&before) && out.ends_with(&after));
                    let after_names: Vec<String> = speakers(&out, "file:p")
                        .iter()
                        .flat_map(|s| std::iter::repeat_n(s.name.clone(), s.paragraphs as usize))
                        .collect();
                    // Order of paragraphs is preserved; compare in order.
                    let in_order: Vec<String> = out
                        .lines()
                        .filter_map(|l| name_range(l).map(|(a, b)| l[a..b].to_owned()))
                        .collect();
                    prop_assert_eq!(in_order.len(), names.len());
                    for (old, now) in names.iter().zip(&in_order) {
                        if *old == from { prop_assert_eq!(now, &new); } else { prop_assert_eq!(now, old); }
                    }
                    prop_assert_eq!(after_names.len(), names.len());
                }
            }
        }

        /// Renaming and renaming back is the identity, when the new name is
        /// not already someone else's.
        #[test]
        fn prop_rename_round_trips(
            who in proptest::collection::vec(0usize..3, 1..10),
            new in "[A-Z][a-z]{3,8}",
        ) {
            let names: Vec<String> = who.iter().map(|w| format!("Speaker {}", w + 1)).collect();
            let body = turns(&names);
            let from = names[0].clone();
            let renamed = rename(&body, "file:p", &from, &new).unwrap();
            prop_assert_eq!(rename(&renamed, "file:p", &new, &from).unwrap(), body);
        }

        /// Applying the changes a form asks for never loses a paragraph and
        /// never touches the prose around the block.
        #[test]
        fn prop_many_changes_keep_every_paragraph(
            who in proptest::collection::vec(0usize..4, 1..12),
            to in proptest::collection::vec("[A-Z][a-z]{2,6}", 4),
            before in "[a-z \n]{0,20}",
            after in "[a-z \n]{0,20}",
        ) {
            let names: Vec<String> = who.iter().map(|w| format!("Speaker {}", w + 1)).collect();
            let body = format!("{before}{}{after}", turns(&names));
            let listed = speakers(&body, "file:p");
            let asked = changes(&listed, &to);
            if let Some(out) = rename_many(&body, "file:p", &asked) {
                prop_assert!(out.starts_with(&before) && out.ends_with(&after));
                let count = |b: &str| b.matches("](kanso-audio://file:p?t=").count();
                prop_assert_eq!(count(&out), count(&body));
                let first = first_recording(&out);
                prop_assert_eq!(first.as_deref(), Some("file:p"));
            }
        }

        /// Transcribing the same recording again, whatever numbers the
        /// voices come out with, gives every paragraph the name it had:
        /// the names follow the voices. Only names move.
        #[test]
        fn prop_a_new_transcript_keeps_given_names(
            who in proptest::collection::vec(0usize..4, 1..12),
            given in proptest::collection::vec("[A-Z][a-z]{2,8}", 4),
            renumber in Just((0usize..4).collect::<Vec<_>>()).prop_shuffle(),
        ) {
            prop_assume!(given.iter().all(|g| !is_ours(g)));
            let label = |w: usize| format!("Speaker {}", w + 1);
            let names: Vec<String> = who.iter().map(|w| label(*w)).collect();
            let asked: Vec<(String, String)> =
                (0..4).map(|w| (label(w), given[w].clone())).collect();
            let named = rename_many(&turns(&names), "file:p", &asked).unwrap();
            let again: Vec<String> = who.iter().map(|w| label(renumber[*w])).collect();
            let fresh = turns(&again);
            let kept = carry_names(&named, &fresh, "file:p").unwrap();
            prop_assert_eq!(&kept, &named);
            // The fresh transcript's words and moments are all still there.
            let strip = |b: &str| -> Vec<String> {
                b.lines()
                    .map(|l| match name_range(l) {
                        Some((a, e)) => format!("{}{}", &l[..a], &l[e..]),
                        None => l.to_owned(),
                    })
                    .collect()
            };
            prop_assert_eq!(strip(&kept), strip(&fresh));
        }

        /// Names Kanso wrote itself never carry over: a transcript nobody
        /// renamed leaves a new one as it came.
        #[test]
        fn prop_kansos_own_labels_never_carry(
            before in proptest::collection::vec(0usize..4, 1..12),
            after in proptest::collection::vec(0usize..4, 1..12),
        ) {
            let label = |w: &usize| format!("Speaker {}", w + 1);
            let old = turns(&before.iter().map(label).collect::<Vec<_>>());
            let fresh = turns(&after.iter().map(label).collect::<Vec<_>>());
            prop_assert_eq!(carry_names(&old, &fresh, "file:p"), None);
        }

        /// A cleaned name is always safe to write: short, one line, no
        /// markup, and cleaning it again changes nothing.
        #[test]
        fn prop_clean_names_are_safe_and_stable(raw in ".{0,80}") {
            if let Some(name) = clean_name(&raw) {
                prop_assert!(name.chars().count() <= MAX_NAME_CHARS);
                prop_assert!(!name.contains(['\n', ':', '[', ']', '<', '>', '(', ')', '`', '*', '_']));
                prop_assert!(looks_like_a_name(&name));
                prop_assert_eq!(clean_name(&name), Some(name));
            }
        }
    }
}
