//! The logic of the Draft, Make Flashcards and Quiz Me pages (issue #198),
//! apart from the window: the cards a person ticks, how a quiz is scored,
//! and how a checked draft is cut into the runs the page underlines.

use kanso_ai::draft_model::{Graded, QuizQuestion, Span, SpanKind, Suggestion, Verdict};

/// Which page is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Draft,
    Make,
    Quiz,
}

impl Mode {
    pub fn title(self) -> &'static str {
        match self {
            Mode::Draft => "Draft from Notes",
            Mode::Make => "Make Flashcards",
            Mode::Quiz => "Quiz Me",
        }
    }
}

// MARK: Suggested cards

/// A suggested card with its tick box.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub question: String,
    pub answer: String,
    /// The quote of the note the card comes from.
    pub source: Option<String>,
    pub line: Option<u32>,
    /// The quote was not found in the note, so the card starts unticked.
    pub missing: bool,
    pub on: bool,
}

/// The picks for `suggestions`: every card ticked except those whose source
/// quote the note does not hold.
pub fn picks_from(suggestions: Vec<Suggestion>) -> Vec<Pick> {
    suggestions
        .into_iter()
        .map(|s| Pick {
            on: !s.source_missing,
            question: s.question,
            answer: s.answer,
            source: s.source,
            line: s.line,
            missing: s.source_missing,
        })
        .collect()
}

/// The (question, answer) pairs to write: the ticked picks, with any edit
/// the person made, and none that came out empty.
pub fn chosen(picks: &[Pick]) -> Vec<(String, String)> {
    picks
        .iter()
        .filter(|p| p.on)
        .map(|p| (p.question.trim().to_owned(), p.answer.trim().to_owned()))
        .filter(|(q, a)| !q.is_empty() && !a.is_empty())
        .collect()
}

/// "3 of 5 cards", for the Add button.
pub fn add_label(picks: &[Pick]) -> String {
    let n = chosen(picks).len();
    match n {
        0 => "Add Cards".to_owned(),
        1 => "Add 1 Card".to_owned(),
        n => format!("Add {n} Cards"),
    }
}

// MARK: Quiz

/// One sitting of Quiz Me: five questions, each answered and graded once.
#[derive(Clone, Debug, Default)]
pub struct Quiz {
    questions: Vec<QuizQuestion>,
    results: Vec<Option<Graded>>,
    at: usize,
}

impl Quiz {
    pub fn new(questions: Vec<QuizQuestion>) -> Quiz {
        Quiz {
            results: vec![None; questions.len()],
            questions,
            at: 0,
        }
    }

    pub fn current(&self) -> Option<&QuizQuestion> {
        self.questions.get(self.at)
    }

    pub fn graded(&self) -> Option<&Graded> {
        self.results.get(self.at).and_then(Option::as_ref)
    }

    pub fn grade(&mut self, graded: Graded) {
        if let Some(slot) = self.results.get_mut(self.at) {
            *slot = Some(graded);
        }
    }

    /// Moves on, if the current question was graded.
    pub fn next(&mut self) {
        if self.graded().is_some() {
            self.at += 1;
        }
    }

    pub fn is_done(&self) -> bool {
        self.at >= self.questions.len()
    }

    /// "Question 2 of 5".
    pub fn progress(&self) -> String {
        format!(
            "Question {} of {}",
            (self.at + 1).min(self.questions.len().max(1)),
            self.questions.len()
        )
    }

    /// (right, partly, wrong) so far.
    pub fn tally(&self) -> (usize, usize, usize) {
        let count = |v: Verdict| {
            self.results
                .iter()
                .flatten()
                .filter(|g| g.verdict == v)
                .count()
        };
        (
            count(Verdict::Right),
            count(Verdict::Partly),
            count(Verdict::Wrong),
        )
    }

    /// "4 right, 1 partly, 0 not quite".
    pub fn summary(&self) -> String {
        let (right, partly, wrong) = self.tally();
        format!("{right} right, {partly} partly, {wrong} not quite")
    }
}

// MARK: Draft runs

/// How a run of a draft is marked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Run {
    Plain,
    /// Backed by a source: drawn plain, with its numbers.
    Cited,
    /// No source in the notes: drawn with a muted underline.
    Unsourced,
}

/// `text` cut into runs by `spans` (byte offsets). Runs cover the text
/// exactly once, in order; spans that overlap, go backwards or fall off a
/// character boundary are cut down to what is left.
pub fn runs(text: &str, spans: &[Span]) -> Vec<(String, Run)> {
    let mut out: Vec<(String, Run)> = Vec::new();
    let mut at = 0usize;
    let mut push = |piece: &str, run: Run| {
        if piece.is_empty() {
            return;
        }
        match out.last_mut() {
            Some((last, kind)) if *kind == run => last.push_str(piece),
            _ => out.push((piece.to_owned(), run)),
        }
    };
    for span in spans {
        let start = span.start.max(at).min(text.len());
        let end = span.end.min(text.len());
        if end <= start || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
            continue;
        }
        push(&text[at..start], Run::Plain);
        let run = match span.kind {
            SpanKind::Cited(_) => Run::Cited,
            SpanKind::Unsourced => Run::Unsourced,
        };
        push(&text[start..end], run);
        at = end;
    }
    push(&text[at..], Run::Plain);
    out
}

/// How many sentences carry no source, for the line above the draft.
pub fn unsourced(spans: &[Span]) -> usize {
    spans
        .iter()
        .filter(|s| matches!(s.kind, SpanKind::Unsourced))
        .count()
}

#[cfg(test)]
mod tests {
    use kanso_ai::draft_model::Cite;
    use proptest::prelude::*;

    use super::*;

    fn suggestion(q: &str, a: &str, missing: bool) -> Suggestion {
        Suggestion {
            question: q.into(),
            answer: a.into(),
            source: Some("quote".into()),
            line: Some(3),
            source_missing: missing,
        }
    }

    #[test]
    fn a_card_whose_quote_is_missing_starts_unticked() {
        let picks = picks_from(vec![
            suggestion("Q1", "A1", false),
            suggestion("Q2", "A2", true),
        ]);
        assert!(picks[0].on);
        assert!(!picks[1].on);
        assert_eq!(chosen(&picks), vec![("Q1".to_owned(), "A1".to_owned())]);
        assert_eq!(add_label(&picks), "Add 1 Card");
    }

    #[test]
    fn an_emptied_card_is_not_written() {
        let mut picks = picks_from(vec![suggestion("Q1", "A1", false)]);
        picks[0].answer = "  ".into();
        assert!(chosen(&picks).is_empty());
        assert_eq!(add_label(&picks), "Add Cards");
    }

    #[test]
    fn a_quiz_moves_on_only_once_graded() {
        let mut quiz = Quiz::new(vec![
            QuizQuestion {
                question: "Q1".into(),
                excerpt: 1,
                source: None,
            },
            QuizQuestion {
                question: "Q2".into(),
                excerpt: 1,
                source: None,
            },
        ]);
        assert_eq!(quiz.progress(), "Question 1 of 2");
        quiz.next();
        assert_eq!(quiz.current().unwrap().question, "Q1", "not graded yet");
        quiz.grade(Graded {
            verdict: Verdict::Right,
            comment: "ok".into(),
        });
        quiz.next();
        assert_eq!(quiz.current().unwrap().question, "Q2");
        quiz.grade(Graded {
            verdict: Verdict::Wrong,
            comment: "no".into(),
        });
        quiz.next();
        assert!(quiz.is_done());
        assert_eq!(quiz.summary(), "1 right, 0 partly, 1 not quite");
    }

    #[test]
    fn runs_mark_the_unsourced_stretch() {
        let spans = vec![
            Span {
                start: 0,
                end: 5,
                kind: SpanKind::Cited(vec![Cite {
                    number: 1,
                    heading: None,
                }]),
            },
            Span {
                start: 6,
                end: 11,
                kind: SpanKind::Unsourced,
            },
        ];
        let out = runs("Hello world!", &spans);
        assert_eq!(
            out,
            vec![
                ("Hello".to_owned(), Run::Cited),
                (" ".to_owned(), Run::Plain),
                ("world".to_owned(), Run::Unsourced),
                ("!".to_owned(), Run::Plain),
            ]
        );
        assert_eq!(unsourced(&spans), 1);
    }

    proptest! {
        /// However the spans fall (overlapping, backwards, inside a
        /// character), the runs put back together are the text.
        #[test]
        fn prop_runs_cover_the_text_exactly(
            text in "[a-zé漢 .\n]{0,60}",
            cuts in prop::collection::vec((0usize..80, 0usize..80, any::<bool>()), 0..8),
        ) {
            let spans: Vec<Span> = cuts
                .into_iter()
                .map(|(start, end, cited)| Span {
                    start,
                    end,
                    kind: if cited {
                        SpanKind::Cited(vec![])
                    } else {
                        SpanKind::Unsourced
                    },
                })
                .collect();
            let joined: String = runs(&text, &spans).into_iter().map(|(s, _)| s).collect();
            prop_assert_eq!(joined, text);
        }
    }
}
