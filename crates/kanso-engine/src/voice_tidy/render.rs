//! Turning a model's tidy-up into the note's Markdown. Pure.
//!
//! Only what the transcript supports is written: a decision or an action
//! item without a quote that is really in the transcript is dropped
//! (counted in [`Rendered::dropped`]), and the transcript itself goes in
//! verbatim, folded, below the tidy part.

use kanso_ai::tidy::{Tidy, TidyKind};

use super::quote::valid_quotes;
use crate::voice_capture::duration_label;

/// What the renderer needs to know besides the model's reply.
#[derive(Clone, Debug)]
pub struct RenderMeta {
    pub kind: TidyKind,
    pub duration_ms: u64,
    /// Whether the words were made on the device ("on this device").
    pub on_device: bool,
    /// The day, for a meeting's heading ("2 Nov 2026").
    pub date: Option<String>,
    /// The title when the model gave none.
    pub fallback_title: String,
}

/// A rendered tidy-up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// The note's title.
    pub title: String,
    /// The tidy part: the heading down to the line before the transcript.
    pub above: String,
    /// The folded transcript block.
    pub details: String,
    /// Decisions and action items left out for want of a quote.
    pub dropped: usize,
}

impl Rendered {
    /// The whole body: tidy part, then the folded transcript.
    pub fn body(&self) -> String {
        format!("{}\n\n{}", self.above, self.details)
    }
}

/// The opening of the folded transcript block; reading a note back finds
/// the transcript by it.
pub const DETAILS_OPEN: &str = "<details><summary>Transcript";
/// The close of that block.
pub const DETAILS_CLOSE: &str = "</details>";

/// `transcript` where it cannot close the block it sits in.
fn escape(transcript: &str) -> String {
    transcript.replace("</details", "&lt;/details")
}

fn unescape(transcript: &str) -> String {
    transcript.replace("&lt;/details", "</details")
}

/// The folded transcript block for `transcript`.
pub fn details_block(transcript: &str, duration_ms: u64, on_device: bool) -> String {
    let place = if on_device {
        "on this device"
    } else {
        "sent to your phone's speech service"
    };
    format!(
        "{DETAILS_OPEN} · {} · {place}</summary>\n\n{}\n{DETAILS_CLOSE}",
        duration_label(duration_ms),
        escape(transcript.trim())
    )
}

/// A note body read back: what's above the transcript block, the
/// transcript, and whatever follows the block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Split<'a> {
    pub above: &'a str,
    pub transcript: String,
    pub below: &'a str,
}

/// Finds the folded transcript block in `body`. `None` when there is none
/// (an untidied capture is just its transcript).
pub fn split_body(body: &str) -> Option<Split<'_>> {
    let open = body.find(DETAILS_OPEN)?;
    let after_open = &body[open..];
    let summary_end = after_open.find("</summary>")? + "</summary>".len();
    let content = &after_open[summary_end..];
    let close = content.rfind(DETAILS_CLOSE)?;
    Some(Split {
        above: body[..open].trim_end(),
        transcript: unescape(content[..close].trim()),
        below: content[close + DETAILS_CLOSE.len()..].trim(),
    })
}

fn action_line(item: &kanso_ai::tidy::Action) -> String {
    let mut line = String::from("- [ ] ");
    if let Some(who) = &item.who {
        line.push_str(who);
        line.push_str(": ");
    }
    line.push_str(&item.text);
    if let Some(when) = &item.when {
        line.push_str(" — ");
        line.push_str(when);
    }
    line
}

/// Renders `tidy` for `transcript`.
///
/// A short note is a heading, paragraphs (with `##` headings where the talk
/// changed topic), then any decisions and tasks. A meeting is `# Title — day`
/// with Summary, Decisions, Action items and Notes, each only if it has
/// something. Never empty: with nothing usable from the model the note is
/// its title and the transcript.
pub fn render_tidy(tidy: &Tidy, transcript: &str, meta: &RenderMeta) -> Rendered {
    let mut dropped = 0;
    let decisions: Vec<&str> = tidy
        .decisions
        .iter()
        .filter(|d| {
            let ok = !valid_quotes(transcript, &d.quotes).is_empty();
            dropped += usize::from(!ok);
            ok
        })
        .map(|d| d.text.as_str())
        .collect();
    let actions: Vec<String> = tidy
        .actions
        .iter()
        .filter(|a| {
            let ok = !valid_quotes(transcript, &a.quotes).is_empty();
            dropped += usize::from(!ok);
            ok
        })
        .map(action_line)
        .collect();

    let name = tidy
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| meta.fallback_title.clone());
    let meeting = meta.kind == TidyKind::Meeting;
    let title = match (&meta.date, meeting) {
        (Some(date), true) => format!("{name} — {date}"),
        _ => name,
    };

    let mut blocks: Vec<String> = vec![format!("# {title}")];
    let section = |blocks: &mut Vec<String>, heading: &str, body: String| {
        if !body.is_empty() {
            blocks.push(format!("## {heading}\n\n{body}"));
        }
    };
    if meeting {
        section(&mut blocks, "Summary", tidy.summary.join("\n\n"));
        section(
            &mut blocks,
            "Decisions",
            decisions
                .iter()
                .map(|d| format!("- {d}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        section(&mut blocks, "Action items", actions.join("\n"));
        let notes: Vec<String> = tidy
            .sections
            .iter()
            .map(|s| {
                let mut part = String::new();
                if !s.heading.is_empty() {
                    part.push_str(&format!("### {}\n\n", s.heading));
                }
                part.push_str(&s.paragraphs.join("\n\n"));
                part
            })
            .collect();
        section(&mut blocks, "Notes", notes.join("\n\n"));
    } else {
        if !tidy.summary.is_empty() {
            blocks.push(tidy.summary.join("\n\n"));
        }
        for s in &tidy.sections {
            if !s.heading.is_empty() {
                blocks.push(format!("## {}", s.heading));
            }
            blocks.push(s.paragraphs.join("\n\n"));
        }
        if !decisions.is_empty() {
            blocks.push(format!(
                "## Decisions\n\n{}",
                decisions
                    .iter()
                    .map(|d| format!("- {d}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }
        if !actions.is_empty() {
            blocks.push(actions.join("\n"));
        }
    }
    if tidy.cut {
        blocks.push(
            "*Only the first four hours of this recording were tidied; the transcript below is whole.*"
                .to_owned(),
        );
    }
    Rendered {
        title,
        above: blocks.join("\n\n"),
        details: details_block(transcript, meta.duration_ms, meta.on_device),
        dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_ai::tidy::{Action, Quoted, Section};
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    fn meta(kind: TidyKind) -> RenderMeta {
        RenderMeta {
            kind,
            duration_ms: 24_000,
            on_device: true,
            date: Some("2 Nov 2026".into()),
            fallback_title: "Voice note".into(),
        }
    }

    const SAID: &str =
        "remind me to call the plumber about the leak on Tuesday and ship the beta on the sixth";

    fn tidy() -> Tidy {
        Tidy {
            title: Some("Plumber".into()),
            summary: vec!["A short list.".into()],
            decisions: vec![
                Quoted {
                    text: "Ship the beta on the 6th".into(),
                    quotes: vec!["ship the beta on the sixth".into()],
                },
                Quoted {
                    text: "Invented".into(),
                    quotes: vec!["never said this".into()],
                },
            ],
            actions: vec![
                Action {
                    text: "Call the plumber about the leak".into(),
                    who: None,
                    when: Some("Tuesday".into()),
                    quotes: vec!["call the plumber about the leak".into()],
                },
                Action {
                    text: "Nothing quoted".into(),
                    who: Some("Priya".into()),
                    when: None,
                    quotes: vec![],
                },
            ],
            sections: vec![Section {
                heading: "Pipes".into(),
                paragraphs: vec!["The leak.".into()],
            }],
            cut: false,
        }
    }

    #[test]
    fn a_note_has_tasks_and_drops_what_it_cannot_quote() {
        let r = render_tidy(&tidy(), SAID, &meta(TidyKind::Note));
        assert_eq!(r.dropped, 2);
        assert!(r.above.starts_with("# Plumber\n"));
        assert!(
            r.above
                .contains("- [ ] Call the plumber about the leak — Tuesday")
        );
        assert!(r.above.contains("- Ship the beta on the 6th"));
        assert!(!r.above.contains("Invented") && !r.above.contains("Nothing quoted"));
        assert!(
            r.details
                .starts_with("<details><summary>Transcript · 0:24 · on this device")
        );
    }

    #[test]
    fn a_meeting_has_the_fixed_shape() {
        let r = render_tidy(&tidy(), SAID, &meta(TidyKind::Meeting));
        assert!(r.above.starts_with("# Plumber — 2 Nov 2026"));
        let order: Vec<usize> = ["## Summary", "## Decisions", "## Action items", "## Notes"]
            .iter()
            .map(|h| r.above.find(h).unwrap())
            .collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        assert!(r.above.contains("### Pipes"));
    }

    #[test]
    fn an_empty_reply_is_the_title_and_the_transcript() {
        let r = render_tidy(&Tidy::default(), SAID, &meta(TidyKind::Meeting));
        assert_eq!(r.above, "# Voice note — 2 Nov 2026");
        assert!(r.body().contains(SAID));
    }

    #[test]
    fn the_body_reads_back() {
        let r = render_tidy(&tidy(), "hi </details> there", &meta(TidyKind::Note));
        let body = format!("{}\n\nAudio link", r.body());
        let split = split_body(&body).unwrap();
        assert_eq!(split.transcript, "hi </details> there");
        assert_eq!(split.above, r.above);
        assert_eq!(split.below, "Audio link");
        assert!(split_body("just words").is_none());
    }

    fn item() -> impl Strategy<Value = (String, Vec<String>)> {
        (
            "[a-z ]{1,12}",
            proptest::collection::vec(
                prop_oneof!["[a-z]{5,10} [a-z]{5,10}", Just(String::new())],
                0..3,
            ),
        )
    }

    proptest! {
        #![proptest_config(Config {
            cases: 128,
            rng_seed: RngSeed::Fixed(0x0072_656e_6465),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        })]

        /// Whatever the model says, the transcript is in the output verbatim
        /// and every decision or task written has a quote that is there.
        #[test]
        fn prop_output_is_grounded_and_keeps_the_transcript(
            transcript in "[a-z ]{20,80}",
            decisions in proptest::collection::vec(item(), 0..4),
            meeting in any::<bool>(),
        ) {
            let kind = if meeting { TidyKind::Meeting } else { TidyKind::Note };
            let t = Tidy {
                decisions: decisions
                    .iter()
                    .map(|(text, quotes)| Quoted { text: format!("D{text}"), quotes: quotes.clone() })
                    .collect(),
                ..Tidy::default()
            };
            let r = render_tidy(&t, &transcript, &meta(kind));
            prop_assert_eq!(split_body(&r.body()).unwrap().transcript, transcript.trim());
            let kept = r.above.lines().filter(|l| l.starts_with("- D")).count();
            let grounded = t
                .decisions
                .iter()
                .filter(|d| !valid_quotes(&transcript, &d.quotes).is_empty())
                .count();
            prop_assert_eq!(kept, grounded);
            prop_assert_eq!(r.dropped, t.decisions.len() - grounded);
        }
    }
}
