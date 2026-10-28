//! Spoken punctuation and commands for voice input: "comma", "new
//! paragraph", "bullet point", "delete that" and the rest, turned from the
//! speech model's words into the text that goes in at the caret
//! ([`dictated`]). The same rules run on the words shown while someone
//! speaks and on the final transcript, so what shows is what lands.
//!
//! The speech model punctuates by itself, and writes a mark where someone
//! pauses. So a command word counts only when it was said on its own:
//! - it is the whole word, in plain letters ("comma-separated" stays);
//! - a phrase of several words ("full stop", "new line") has no mark inside
//!   it;
//! - and the commands said together sit between pauses: the model wrote a
//!   mark (`.,;:!?…`) right before the first and right after the last, or
//!   they start or end what was said.
//!
//! "The period of time" and "a comma between them" stay as words, while
//! "Buy milk. Comma. Eggs." becomes "Buy milk, eggs.". The mark the model
//! wrote for the pause before a punctuation command gives way to the one
//! said; before a layout command ("new line") it stays, as it may well end
//! a sentence.
//!
//! "Delete that" and "scratch that" take out the last phrase said (up to
//! the model's last mark before it) and anything said after it; "undo that"
//! takes back the last step, whether words, a mark or a delete. Both reach
//! only what was said this time: the text already in the note is never
//! changed.
//!
//! Spacing and capitals follow the marks: no space before `.,;:!?` or a
//! closing quote or bracket, none after an opening one; a capital after a
//! full stop, question or exclamation mark, a new paragraph or a new list
//! item, and a small letter after a comma, semicolon or dash (except "I"
//! and words in capitals).

/// A mark or layout said aloud.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    FullStop,
    Comma,
    Question,
    Exclamation,
    Colon,
    Semicolon,
    OpenQuote,
    CloseQuote,
    OpenBracket,
    CloseBracket,
    Dash,
    NewLine,
    NewParagraph,
    /// Starts a `- ` list item.
    Bullet,
    /// Starts a `- [ ] ` to-do.
    Todo,
}

/// What a command phrase does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Mark(Mark),
    /// "Delete that", "scratch that".
    Delete,
    /// "Undo that".
    Undo,
}

/// The phrases, in the model's spelling, lowercased.
const PHRASES: &[(&[&str], Command)] = {
    use Command::{Delete, Mark as M, Undo};
    use Mark::*;
    &[
        (&["new", "line"], M(NewLine)),
        (&["new", "paragraph"], M(NewParagraph)),
        (&["full", "stop"], M(FullStop)),
        (&["period"], M(FullStop)),
        (&["comma"], M(Comma)),
        (&["question", "mark"], M(Question)),
        (&["exclamation", "mark"], M(Exclamation)),
        (&["exclamation", "point"], M(Exclamation)),
        (&["colon"], M(Colon)),
        (&["semicolon"], M(Semicolon)),
        (&["semi", "colon"], M(Semicolon)),
        (&["open", "quote"], M(OpenQuote)),
        (&["close", "quote"], M(CloseQuote)),
        (&["open", "bracket"], M(OpenBracket)),
        (&["close", "bracket"], M(CloseBracket)),
        (&["dash"], M(Dash)),
        (&["bullet", "point"], M(Bullet)),
        (&["new", "to-do"], M(Todo)),
        (&["new", "todo"], M(Todo)),
        (&["new", "to", "do"], M(Todo)),
        (&["checkbox"], M(Todo)),
        (&["check", "box"], M(Todo)),
        (&["delete", "that"], Delete),
        (&["scratch", "that"], Delete),
        (&["undo", "that"], Undo),
    ]
};

/// The marks the model writes where someone pauses.
const PAUSES: &[char] = &['.', ',', ';', ':', '!', '?', '…'];

/// Characters after which words follow with no space.
const OPENERS: &str = "([{\"'“‘";

/// The text to put in for `transcript`, between `before` (the note's text
/// up to the caret) and the character `after` it, and where the caret goes
/// in it. With `commands` off, the transcript goes in as said. `None` when
/// nothing is left to put in.
pub fn dictated(
    before: &str,
    after: Option<char>,
    transcript: &str,
    commands: bool,
) -> Option<(String, usize)> {
    let items = if commands {
        apply(&steps(transcript))
    } else {
        let words = transcript.trim();
        if words.is_empty() {
            Vec::new()
        } else {
            vec![Item::Words {
                text: words.to_owned(),
                sep: None,
            }]
        }
    };
    let out = render(before, after, &items);
    (!out.0.is_empty()).then_some(out)
}

/// One whitespace-separated word of the transcript.
struct Token<'a> {
    start: usize,
    end: usize,
    /// The word without the marks after it.
    word: &'a str,
    /// The model wrote a pause mark after it.
    paused: bool,
}

fn tokens(transcript: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut at = 0;
    for raw in transcript.split_whitespace() {
        let start = at + transcript[at..].find(raw).unwrap_or(0);
        let end = start + raw.len();
        at = end;
        let word = raw.trim_end_matches(PAUSES);
        out.push(Token {
            start,
            end,
            word,
            paused: word.len() < raw.len(),
        });
    }
    out
}

/// The longest command phrase starting at `tokens[at]`, and its length.
fn command_at(tokens: &[Token], at: usize) -> Option<(usize, Command)> {
    PHRASES
        .iter()
        .filter(|(words, _)| {
            let Some(said) = tokens.get(at..at + words.len()) else {
                return false;
            };
            said.iter()
                .zip(words.iter())
                .enumerate()
                .all(|(i, (t, w))| {
                    let last = i + 1 == words.len();
                    (last || !t.paused) && t.word.eq_ignore_ascii_case(w)
                })
        })
        .map(|(words, command)| (words.len(), *command))
        .max_by_key(|(len, _)| *len)
}

/// What was said, in order: phrases of words and the commands that count.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Step<'a> {
    /// Words up to a pause; `sep` is the space before them when they carry
    /// on the words before, as said.
    Phrase {
        text: &'a str,
        sep: Option<&'a str>,
    },
    Command(Command),
}

fn steps(transcript: &str) -> Vec<Step<'_>> {
    let tokens = tokens(transcript);
    let n = tokens.len();
    // Which tokens are commands, and the command each run starts.
    let mut commands: Vec<Option<Option<Command>>> = vec![None; n];
    let mut i = 0;
    while i < n {
        let edge_before = i == 0 || tokens[i - 1].paused;
        let mut run = Vec::new();
        let mut j = i;
        if edge_before {
            while let Some((len, command)) = command_at(&tokens, j) {
                run.push((j, len, command));
                j += len;
                if tokens[j - 1].paused {
                    break;
                }
            }
        }
        if run.is_empty() || !(j == n || tokens[j - 1].paused) {
            i += 1;
            continue;
        }
        for (at, len, command) in run {
            commands[at] = Some(Some(command));
            for slot in &mut commands[at + 1..at + len] {
                *slot = Some(None);
            }
        }
        i = j;
    }
    let mut out = Vec::new();
    // The phrase being built: where it starts, and the space before it.
    let mut phrase: Option<(usize, Option<&str>)> = None;
    for (i, token) in tokens.iter().enumerate() {
        match commands[i] {
            Some(Some(command)) => out.push(Step::Command(command)),
            Some(None) => {}
            None => {
                let (start, sep) = *phrase.get_or_insert((token.start, None));
                let ends = token.paused || i + 1 == n || commands[i + 1].is_some();
                if ends {
                    out.push(Step::Phrase {
                        text: &transcript[start..token.end],
                        sep,
                    });
                    phrase = (i + 1 < n && commands[i + 1].is_none()).then(|| {
                        (
                            tokens[i + 1].start,
                            Some(&transcript[token.end..tokens[i + 1].start]),
                        )
                    });
                }
            }
        }
    }
    out
}

/// What goes in, once the commands have acted.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Item {
    Words { text: String, sep: Option<String> },
    Mark(Mark),
}

/// How to take a step back.
enum Back {
    /// Take the last item off, and give the words before it back the
    /// marks a punctuation command took from them.
    Pop(Option<String>),
    /// Put back what a delete took.
    Push(Vec<Item>),
}

fn apply(steps: &[Step]) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    let mut back: Vec<Back> = Vec::new();
    for step in steps {
        match *step {
            Step::Phrase { text, sep } => {
                items.push(Item::Words {
                    text: text.to_owned(),
                    sep: sep.map(str::to_owned),
                });
                back.push(Back::Pop(None));
            }
            Step::Command(Command::Mark(mark)) => {
                let mut restore = None;
                if mark.replaces_pause()
                    && let Some(Item::Words { text, .. }) = items.last_mut()
                {
                    let kept = text.trim_end_matches(PAUSES).len();
                    if kept > 0 && kept < text.len() {
                        restore = Some(text.clone());
                        text.truncate(kept);
                    }
                }
                items.push(Item::Mark(mark));
                back.push(Back::Pop(restore));
            }
            Step::Command(Command::Delete) => {
                let from = items
                    .iter()
                    .rposition(|item| matches!(item, Item::Words { .. }))
                    .unwrap_or(0);
                let removed = items.split_off(from);
                if !removed.is_empty() {
                    back.push(Back::Push(removed));
                }
            }
            Step::Command(Command::Undo) => match back.pop() {
                Some(Back::Pop(restore)) => {
                    items.pop();
                    if let (Some(old), Some(Item::Words { text, .. })) = (restore, items.last_mut())
                    {
                        *text = old;
                    }
                }
                Some(Back::Push(removed)) => items.extend(removed),
                None => {}
            },
        }
    }
    items
}

impl Mark {
    /// Whether the mark takes the place of the model's own mark for the
    /// pause before it.
    fn replaces_pause(self) -> bool {
        matches!(
            self,
            Mark::FullStop
                | Mark::Comma
                | Mark::Question
                | Mark::Exclamation
                | Mark::Colon
                | Mark::Semicolon
                | Mark::Dash
        )
    }

    /// The case the next words take: `Some(true)` a capital, `Some(false)`
    /// a small letter, `None` as said.
    fn case_after(self) -> Option<bool> {
        match self {
            Mark::FullStop
            | Mark::Question
            | Mark::Exclamation
            | Mark::NewParagraph
            | Mark::Bullet
            | Mark::Todo => Some(true),
            Mark::Comma | Mark::Semicolon | Mark::Dash => Some(false),
            _ => None,
        }
    }
}

/// The case the next words take.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    /// The first words: small when they carry on a sentence in the note.
    Context,
    Keep,
    Upper,
    Lower,
}

fn render(before: &str, after: Option<char>, items: &[Item]) -> (String, usize) {
    let mut out = String::new();
    let mut space = before
        .chars()
        .next_back()
        .is_some_and(|c| !c.is_whitespace() && !OPENERS.contains(c));
    let mut case = Case::Context;
    let mut prev_words = false;
    for item in items {
        match item {
            Item::Words { text, sep } => {
                if let (Some(sep), true) = (sep, prev_words) {
                    out.push_str(sep);
                    out.push_str(text);
                } else {
                    if space {
                        out.push(' ');
                    }
                    let lower = match case {
                        Case::Context => mid_sentence(&format!("{before}{out}")),
                        Case::Lower => true,
                        Case::Keep | Case::Upper => false,
                    };
                    if lower {
                        out.push_str(&lowercase_first(text));
                    } else if case == Case::Upper {
                        out.push_str(&uppercase_first(text));
                    } else {
                        out.push_str(text);
                    }
                }
                space = true;
                case = Case::Keep;
                prev_words = true;
            }
            Item::Mark(mark) => {
                let line_start = out
                    .chars()
                    .next_back()
                    .or_else(|| before.chars().next_back())
                    .is_none_or(|c| c == '\n');
                let (spaced, text, space_after) = match mark {
                    Mark::FullStop => (false, ".", true),
                    Mark::Comma => (false, ",", true),
                    Mark::Question => (false, "?", true),
                    Mark::Exclamation => (false, "!", true),
                    Mark::Colon => (false, ":", true),
                    Mark::Semicolon => (false, ";", true),
                    Mark::CloseQuote => (false, "\"", true),
                    Mark::CloseBracket => (false, ")", true),
                    Mark::OpenQuote => (true, "\"", false),
                    Mark::OpenBracket => (true, "(", false),
                    Mark::Dash => (true, "–", true),
                    Mark::NewLine => (false, "\n", false),
                    Mark::NewParagraph => (false, "\n\n", false),
                    Mark::Bullet if line_start => (false, "- ", false),
                    Mark::Bullet => (false, "\n- ", false),
                    Mark::Todo if line_start => (false, "- [ ] ", false),
                    Mark::Todo => (false, "\n- [ ] ", false),
                };
                if spaced && space {
                    out.push(' ');
                }
                out.push_str(text);
                space = space_after;
                match mark.case_after() {
                    Some(true) => case = Case::Upper,
                    Some(false) => case = Case::Lower,
                    None => {}
                }
                prev_words = false;
            }
        }
    }
    let caret = out.len();
    if !out.is_empty() && space && after.is_some_and(char::is_alphanumeric) {
        out.push(' ');
    }
    (out, caret)
}

/// Whether text that follows `before` carries on a sentence: something
/// came before on the line, and it didn't end the sentence.
pub fn mid_sentence(before: &str) -> bool {
    let line = before.rsplit('\n').next().unwrap_or("");
    let trimmed = line.trim_end();
    // A list item's or heading's marker starts a sentence.
    let content = trimmed
        .trim_start_matches(['#', '-', '*', '+', '>', ' '])
        .trim_end_matches(['(', '[', '{', '"', '\'', '“', '‘']);
    match content.chars().next_back() {
        None => false,
        Some(c) => !".!?:".contains(c) && !content.ends_with("[ ]") && !content.ends_with("[x]"),
    }
}

/// "Hello there" → "hello there"; "I think" and "NASA said" stay.
pub fn lowercase_first(words: &str) -> String {
    let first = words.split_whitespace().next().unwrap_or("");
    let mut chars = first.chars();
    let (Some(a), b) = (chars.next(), chars.next()) else {
        return words.to_owned();
    };
    let keep = first == "I" || first.starts_with("I'") || b.is_some_and(char::is_uppercase);
    if keep || !a.is_uppercase() || !words.starts_with(a) {
        return words.to_owned();
    }
    let mut out: String = a.to_lowercase().collect();
    out.push_str(&words[a.len_utf8()..]);
    out
}

/// "hello there" → "Hello there".
fn uppercase_first(words: &str) -> String {
    let mut chars = words.chars();
    match chars.next() {
        Some(a) if a.is_lowercase() => a.to_uppercase().chain(chars).collect(),
        _ => words.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// What goes in for `transcript` at the start of an empty note.
    fn said(transcript: &str) -> String {
        dictated("", None, transcript, true)
            .map(|(text, _)| text)
            .unwrap_or_default()
    }

    /// What the note reads after `transcript` goes in after `before`.
    fn after(before: &str, transcript: &str) -> String {
        let (text, _) = dictated(before, None, transcript, true).unwrap_or_default();
        format!("{before}{text}")
    }

    #[test]
    fn punctuation_said_on_its_own_is_written() {
        assert_eq!(said("Buy milk. Comma. Eggs."), "Buy milk, eggs.");
        assert_eq!(said("Is it done? Question mark."), "Is it done?");
        assert_eq!(said("Hello, full stop. thanks."), "Hello. Thanks.");
        assert_eq!(said("Hello. Period. thanks"), "Hello. Thanks");
        assert_eq!(said("Wow, exclamation mark."), "Wow!");
        assert_eq!(said("Wow. Exclamation point."), "Wow!");
        assert_eq!(said("Note, colon. Bring tea."), "Note: Bring tea.");
        assert_eq!(said("One, semicolon. Two."), "One; two.");
        assert_eq!(said("One, semi colon, two."), "One; two.");
        assert_eq!(said("Wait. Dash. Not yet."), "Wait – not yet.");
        assert_eq!(
            said("She said, open quote. Hi. Close quote. And left."),
            "She said, \"Hi.\" And left."
        );
        assert_eq!(
            said("Tea. Open bracket. Green. Close bracket."),
            "Tea. (Green.)"
        );
        assert_eq!(said("Comma."), ",");
    }

    #[test]
    fn layout_said_on_its_own_is_written() {
        assert_eq!(said("Dear Sam. New line. Hello."), "Dear Sam.\nHello.");
        assert_eq!(
            said("First thing. New paragraph. second thing."),
            "First thing.\n\nSecond thing."
        );
        assert_eq!(
            said("Shopping. Bullet point. milk. Bullet point. eggs."),
            "Shopping.\n- Milk.\n- Eggs."
        );
        assert_eq!(said("Bullet point. Milk."), "- Milk.");
        assert_eq!(said("New to-do. Call Sam."), "- [ ] Call Sam.");
        assert_eq!(said("New todo, call Sam."), "- [ ] Call Sam.");
        assert_eq!(said("New to do. Call Sam."), "- [ ] Call Sam.");
        assert_eq!(said("Checkbox. Call Sam."), "- [ ] Call Sam.");
        assert_eq!(
            said("Plans. Check box. Call Sam."),
            "Plans.\n- [ ] Call Sam."
        );
        // Commands said together, between pauses.
        assert_eq!(
            said("That's all, full stop new paragraph. Next."),
            "That's all.\n\nNext."
        );
        assert_eq!(said("Hello. New line new line. Bye."), "Hello.\n\nBye.");
    }

    #[test]
    fn command_words_in_a_sentence_stay_words() {
        for words in [
            "The period of time was short.",
            "A comma-separated list.",
            "Put a comma between them.",
            "Draw a dash here.",
            "The new line of products, launched today.",
            "Undo that knot, please.",
            "I'll scratch that itch.",
            "Delete that file now.",
            "No exceptions, period of grace.",
            "A full stop sign.",
            "That's a question mark tattoo.",
            "Comma-free.",
            "\"Comma.\"",
        ] {
            assert_eq!(said(words), words, "{words}");
        }
    }

    #[test]
    fn delete_and_undo_reach_only_what_was_said() {
        assert_eq!(said("Buy milk. Call mom. Delete that."), "Buy milk.");
        assert_eq!(
            said("Buy milk. Call mom. Scratch that. Call dad."),
            "Buy milk. Call dad."
        );
        assert_eq!(said("Buy milk, eggs, delete that."), "Buy milk,");
        assert_eq!(said("Buy milk. Comma. Delete that."), "");
        assert_eq!(said("Buy milk. Comma. Undo that."), "Buy milk.");
        assert_eq!(said("Buy milk. Call mom. Undo that."), "Buy milk.");
        assert_eq!(
            said("Buy milk. Call mom. Delete that. Undo that."),
            "Buy milk. Call mom."
        );
        assert_eq!(said("Delete that. Undo that. Delete that."), "");
        assert_eq!(dictated("Typed words.", None, "Delete that.", true), None);
        assert_eq!(dictated("Typed words.", None, "Undo that.", true), None);
        assert_eq!(
            after("Typed. ", "Hi. Delete that. Delete that. Bye."),
            "Typed. Bye."
        );
    }

    #[test]
    fn the_note_around_decides_spacing_and_case() {
        assert_eq!(after("I said", "Comma. Hello."), "I said, hello.");
        assert_eq!(after("I said", "Hello. Comma. Bye."), "I said hello, bye.");
        assert_eq!(
            after("Groceries", "Bullet point. Milk."),
            "Groceries\n- Milk."
        );
        assert_eq!(
            after("Groceries\n", "Bullet point. Milk."),
            "Groceries\n- Milk."
        );
        assert_eq!(after("Done.", "New line. Next."), "Done.\nNext.");
        assert_eq!(after("(", "Aside."), "(Aside.");
        // The caret goes before the space that keeps words apart.
        let (text, caret) = dictated("one", Some('t'), "Two. Comma.", true).unwrap();
        assert_eq!((text.as_str(), caret), (" two, ", 5));
        let (text, caret) = dictated("one", Some('t'), "Two. New line.", true).unwrap();
        assert_eq!((text.as_str(), caret), (" two.\n", 6));
    }

    #[test]
    fn with_commands_off_the_words_go_in_as_said() {
        let (text, _) = dictated("", None, "Buy milk. Comma. Eggs.", false).unwrap();
        assert_eq!(text, "Buy milk. Comma. Eggs.");
        assert_eq!(dictated("", None, "  ", false), None);
        assert_eq!(dictated("", None, "  ", true), None);
    }

    // Generators.

    /// Words that are never commands.
    fn word() -> impl Strategy<Value = String> {
        prop_oneof![
            proptest::string::string_regex("[a-z]{1,7}").unwrap(),
            Just("I".to_owned()),
            Just("NASA".to_owned()),
            Just("café".to_owned()),
            Just("comma-separated".to_owned()),
            Just("periods".to_owned()),
        ]
        .prop_filter("not a command word", |w| {
            !PHRASES.iter().any(|(words, _)| words.contains(&w.as_str()))
        })
    }

    /// A transcript the way the model writes one, with no command in it:
    /// command words only inside sentences.
    fn plain_transcript() -> impl Strategy<Value = String> {
        let sentence = (
            proptest::collection::vec(word(), 1..6),
            proptest::option::of(0usize..5),
            prop_oneof![Just("."), Just(","), Just("?"), Just("")],
        )
            .prop_map(|(mut words, inner, mark)| {
                if let Some(at) = inner {
                    // A command word, but with words either side.
                    let command = ["comma", "period", "new line", "dash", "delete that"][at];
                    let at = words.len().div_ceil(2).max(1);
                    words.insert(at, command.to_owned());
                    words.push("then".to_owned());
                }
                format!("{}{mark}", words.join(" "))
            });
        proptest::collection::vec(sentence, 1..5).prop_map(|s| s.join(" "))
    }

    /// A part of a transcript, as the model writes it.
    #[derive(Clone, Debug)]
    enum Part {
        /// A phrase, starting with the unique word `w{n}x`.
        Phrase(usize, Vec<String>),
        Command(&'static str, Command),
    }

    fn part() -> impl Strategy<Value = Part> {
        let commands: Vec<(&'static str, Command)> = PHRASES
            .iter()
            .map(|(words, command)| {
                let said: &'static str = Box::leak(words.join(" ").into_boxed_str());
                (said, *command)
            })
            .collect();
        prop_oneof![
            3 => (0usize..1000, proptest::collection::vec(word(), 0..4))
                .prop_map(|(n, words)| Part::Phrase(n, words)),
            2 => proptest::sample::select(commands).prop_map(|(s, c)| Part::Command(s, c)),
        ]
    }

    /// The parts as the model writes them: each ends in a pause mark, and
    /// starts with a capital after a full stop.
    fn transcript_of(parts: &[Part], marks: &[bool]) -> String {
        let mut out = String::new();
        let mut capital = true;
        for (i, part) in parts.iter().enumerate() {
            let words = match part {
                Part::Phrase(n, words) => {
                    let mut all = vec![format!("w{n}x")];
                    all.extend(words.iter().cloned());
                    all.join(" ")
                }
                Part::Command(said, _) => (*said).to_owned(),
            };
            let words = if capital {
                uppercase_first(&words)
            } else {
                words
            };
            let full_stop = marks.get(i).copied().unwrap_or(true);
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&words);
            out.push(if full_stop { '.' } else { ',' });
            capital = full_stop;
        }
        out
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

        /// A transcript with no command in it goes in exactly as it would
        /// with commands off, wherever it lands.
        #[test]
        fn prop_no_commands_no_change(
            transcript in plain_transcript(),
            before in "[A-Za-z .,:\n\\-]{0,12}",
            next in proptest::option::of("[a-z .]"),
        ) {
            let next = next.and_then(|s| s.chars().next());
            prop_assert_eq!(
                dictated(&before, next, &transcript, true),
                dictated(&before, next, &transcript, false)
            );
            prop_assert_eq!(said(&transcript), transcript.trim());
        }

        /// What goes in, said again, goes in the same: the commands acted,
        /// and what is left, command words in sentences too, reads as
        /// words.
        #[test]
        fn prop_idempotent(
            parts in proptest::collection::vec(part(), 0..10),
            marks in proptest::collection::vec(any::<bool>(), 10),
            plain in plain_transcript(),
            at in 0usize..10,
        ) {
            let mut transcript = transcript_of(&parts[..at.min(parts.len())], &marks);
            transcript = format!("{transcript} {plain} {}", transcript_of(&parts[at.min(parts.len())..], &marks));
            let once = said(&transcript);
            let twice = said(&once);
            prop_assert_eq!(twice.trim(), once.trim());
        }

        /// Whatever is said, the caret lands inside what goes in, and
        /// commands said on their own leave no command words behind: only
        /// the words said are in it.
        #[test]
        fn prop_commands_never_reach_the_note(
            parts in proptest::collection::vec(part(), 0..10),
            marks in proptest::collection::vec(any::<bool>(), 10),
            before in "[A-Za-z .,:\n\\-]{0,12}",
            next in proptest::option::of("[a-z .]"),
        ) {
            let transcript = transcript_of(&parts, &marks);
            let next = next.and_then(|s| s.chars().next());
            let undoes = parts.iter().any(|p| matches!(p, Part::Command(_, Command::Delete | Command::Undo)));
            match dictated(&before, next, &transcript, true) {
                Some((text, caret)) => {
                    prop_assert!(caret <= text.len() && text.is_char_boundary(caret));
                    prop_assert!(text[caret..].chars().all(|c| c == ' '));
                    let said_words: std::collections::HashSet<String> = parts
                        .iter()
                        .flat_map(|p| match p {
                            Part::Phrase(n, words) => {
                                let mut all = vec![format!("w{n}x")];
                                all.extend(words.iter().cloned());
                                all
                            }
                            Part::Command(..) => Vec::new(),
                        })
                        .map(|w| w.to_lowercase())
                        .collect();
                    let lower = text.to_lowercase();
                    for w in lower
                        .split(|c: char| !c.is_alphanumeric() && c != '-')
                        .filter(|w| w.chars().any(char::is_alphanumeric))
                    {
                        prop_assert!(said_words.contains(w), "{w} in {text:?}");
                    }
                }
                // Nothing to put in: nothing was said, or deletes and undos
                // took it all.
                None => prop_assert!(
                    undoes
                        || parts.iter().all(|p| !matches!(p, Part::Phrase(..) | Part::Command(_, Command::Mark(_))))
                ),
            }
        }

        /// The words after a command take its case and spacing: a capital
        /// after a sentence's end, a new paragraph or list item; a small
        /// letter after a comma, semicolon or dash; nothing as said after
        /// the others.
        #[test]
        fn prop_case_and_spacing_follow_the_marks(
            parts in proptest::collection::vec(part(), 1..10),
            marks in proptest::collection::vec(any::<bool>(), 10),
        ) {
            let parts: Vec<Part> = parts
                .into_iter()
                .filter(|p| !matches!(p, Part::Command(_, Command::Delete | Command::Undo)))
                .enumerate()
                .map(|(i, p)| match p {
                    Part::Phrase(_, words) => Part::Phrase(i, words),
                    other => other,
                })
                .collect();
            let text = said(&transcript_of(&parts, &marks));
            for pair in parts.windows(2) {
                let (Part::Command(_, Command::Mark(mark)), Part::Phrase(n, _)) = (&pair[0], &pair[1]) else {
                    continue;
                };
                let small = format!("w{n}x");
                let lead = match mark.case_after() {
                    Some(true) => format!("W{n}x"),
                    Some(false) => small.clone(),
                    None => continue,
                };
                let at = text.find(&lead);
                prop_assert!(at.is_some(), "{lead} in {text:?}");
                let before = &text[..at.unwrap()];
                let joins = match mark {
                    Mark::FullStop => ". ",
                    Mark::Question => "? ",
                    Mark::Exclamation => "! ",
                    Mark::Comma => ", ",
                    Mark::Semicolon => "; ",
                    Mark::Dash => "– ",
                    Mark::NewParagraph => "\n\n",
                    Mark::Bullet => "- ",
                    Mark::Todo => "- [ ] ",
                    _ => "",
                };
                prop_assert!(before.ends_with(joins), "{joins:?} before {lead} in {text:?}");
            }
        }

        /// "Delete that" takes off the last phrase, and "undo that" after it
        /// puts it back; "undo that" after words takes them off.
        #[test]
        fn prop_delete_and_undo_round_trip(
            n in 1usize..6,
            words in proptest::collection::vec(word(), 1..4),
        ) {
            let phrases: Vec<String> = (0..n).map(|i| format!("W{i}x {}.", words.join(" "))).collect();
            let all = phrases.join(" ");
            let fewer = phrases[..n - 1].join(" ");
            prop_assert_eq!(said(&format!("{all} Delete that.")), said(&fewer));
            prop_assert_eq!(said(&format!("{all} Undo that.")), said(&fewer));
            prop_assert_eq!(said(&format!("{all} Delete that. Undo that.")), said(&all));
            let many = " Scratch that.".repeat(n + 2);
            prop_assert_eq!(dictated("Typed.", None, &format!("{all}{many}"), true), None);
        }
    }
}
