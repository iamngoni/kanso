use super::*;
use proptest::prelude::*;

/// Labelled sentences: `Some(kind)` when the rules should find a loop.
const CORPUS: &[(&str, Option<LoopKind>)] = &[
    // Mine
    ("I'll send Tendai the deck.", Some(LoopKind::Mine)),
    ("I’ll send Tendai the deck.", Some(LoopKind::Mine)),
    ("I will call the bank on Monday.", Some(LoopKind::Mine)),
    ("I need to book the flights.", Some(LoopKind::Mine)),
    ("I have to renew my passport.", Some(LoopKind::Mine)),
    ("I must reply to Anesu.", Some(LoopKind::Mine)),
    ("I should email the landlord.", Some(LoopKind::Mine)),
    ("Need to check the lease.", Some(LoopKind::Mine)),
    ("need to pay the electricity bill", Some(LoopKind::Mine)),
    ("Have to finish the slides by Friday.", Some(LoopKind::Mine)),
    ("Must submit the form today.", Some(LoopKind::Mine)),
    ("Remember to water the plants.", Some(LoopKind::Mine)),
    ("Don't forget to back up the laptop.", Some(LoopKind::Mine)),
    ("Don’t forget to lock the shed.", Some(LoopKind::Mine)),
    ("Follow up with Rudo about the invoice.", Some(LoopKind::Mine)),
    ("Follow up on the insurance claim.", Some(LoopKind::Mine)),
    ("TODO: renew passport", Some(LoopKind::Mine)),
    ("TODO call the dentist", Some(LoopKind::Mine)),
    ("FIXME the margins on the print view", Some(LoopKind::Mine)),
    ("To do: order the new filters", Some(LoopKind::Mine)),
    ("Action: share the minutes with the team", Some(LoopKind::Mine)),
    ("And I'll book the venue as well.", Some(LoopKind::Mine)),
    ("Also need to update the budget sheet.", Some(LoopKind::Mine)),
    ("I need to sort out the visa @2026-12-01.", Some(LoopKind::Mine)),
    ("I'll draft the proposal tomorrow.", Some(LoopKind::Mine)),
    ("I will review Chipo's pull request.", Some(LoopKind::Mine)),
    ("I need to buy milk, eggs and bread.", Some(LoopKind::Mine)),
    ("I have to pick up Tino from school.", Some(LoopKind::Mine)),
    ("Need to ring the plumber again.", Some(LoopKind::Mine)),
    ("Remember to send the receipts to accounts.", Some(LoopKind::Mine)),
    ("I should really call mum this weekend.", Some(LoopKind::Mine)),
    ("I'll look into the flaky test.", Some(LoopKind::Mine)),
    ("I will write the summary after lunch.", Some(LoopKind::Mine)),
    ("Must remember the parcel slip.", Some(LoopKind::Mine)),
    ("Need to renew the domain.", Some(LoopKind::Mine)),
    ("I'll ask the team about the rota.", Some(LoopKind::Mine)),
    ("Don't forget to vote on Saturday.", Some(LoopKind::Mine)),
    ("I need to figure out the tax return.", Some(LoopKind::Mine)),
    ("I have to take the car in for a service.", Some(LoopKind::Mine)),
    ("I must file the quarterly report.", Some(LoopKind::Mine)),
    ("Follow up with the recruiter next week.", Some(LoopKind::Mine)),
    ("TODO: move the backups to the new drive", Some(LoopKind::Mine)),
    ("I'll bring the projector.", Some(LoopKind::Mine)),
    ("I need to rewrite the intro.", Some(LoopKind::Mine)),
    ("Need to talk to Farai about the roadmap.", Some(LoopKind::Mine)),
    ("I'll set up the staging server.", Some(LoopKind::Mine)),
    ("I will cancel the old subscription.", Some(LoopKind::Mine)),
    ("Remember to book the dentist.", Some(LoopKind::Mine)),
    ("I should renew the car insurance.", Some(LoopKind::Mine)),
    ("I have to chase the courier.", Some(LoopKind::Mine)),
    ("I need to clear the inbox.", Some(LoopKind::Mine)),
    ("Need to pick a gift for Amai.", Some(LoopKind::Mine)),
    ("Waiting on the plumber to show up.", Some(LoopKind::Waiting)),
    // Waiting
    ("Waiting on Rudo for the quote.", Some(LoopKind::Waiting)),
    ("waiting for the landlord to call back", Some(LoopKind::Waiting)),
    ("Still waiting on the visa decision.", Some(LoopKind::Waiting)),
    ("I'm waiting on the bank.", Some(LoopKind::Waiting)),
    ("I am waiting for Tatenda's review.", Some(LoopKind::Waiting)),
    ("Tendai will send the contract on Friday.", Some(LoopKind::Waiting)),
    ("Rudo to confirm the venue.", Some(LoopKind::Waiting)),
    ("Anesu will get back to me about the date.", Some(LoopKind::Waiting)),
    ("Pending from finance: the signed PO.", Some(LoopKind::Waiting)),
    ("Chipo to reply by Tuesday.", Some(LoopKind::Waiting)),
    ("Waiting for the parts to arrive.", Some(LoopKind::Waiting)),
    ("Farai will send the photos tonight.", Some(LoopKind::Waiting)),
    ("Nyasha to send the draft.", Some(LoopKind::Waiting)),
    ("Waiting on legal.", Some(LoopKind::Waiting)),
    ("Tino will confirm the time.", Some(LoopKind::Waiting)),
    ("Waiting on the customer's approval.", Some(LoopKind::Waiting)),
    ("Still waiting for a reply from Kuda.", Some(LoopKind::Waiting)),
    ("Pending from the printer.", Some(LoopKind::Waiting)),
    ("Blessing will respond after the holiday.", Some(LoopKind::Waiting)),
    ("Waiting for Tapiwa to merge the branch.", Some(LoopKind::Waiting)),
    // Questions
    ("Check whether the invoice was paid?", Some(LoopKind::Question)),
    ("Check if the warranty covers water damage?", Some(LoopKind::Question)),
    ("Find out what time the shop closes?", Some(LoopKind::Question)),
    ("Ask Tendai who has the spare key?", Some(LoopKind::Question)),
    ("Find out if the gym has parking", Some(LoopKind::Question)),
    ("Ask Rudo about the quote", Some(LoopKind::Question)),
    ("Check whether the deposit cleared", Some(LoopKind::Question)),
    ("Check if the build is green?", Some(LoopKind::Question)),
    ("Find out which airline allows two bags", Some(LoopKind::Question)),
    ("Ask Chipo whether Friday works?", Some(LoopKind::Question)),
    // Negatives: look-alikes
    ("I needed to leave early yesterday.", None),
    ("I don't need to renew the passport.", None),
    ("There is no need to call the bank.", None),
    ("I will not be at the meeting.", None),
    ("I'll never forgive that referee.", None),
    ("I should have called earlier.", None),
    ("It must be nice to travel so much.", None),
    ("He said \"I'll send the deck tomorrow\" and left.", None),
    ("She wrote “I need to book flights” on the board.", None),
    ("`I'll send` is a string in the test.", None),
    ("The mustard is in the fridge.", None),
    ("Mustard seeds need to soak overnight.", None),
    ("Rudo said the quote is ready.", None),
    ("We waited for the bus for an hour.", None),
    ("The waiting room was full.", None),
    ("Waiting is the hardest part.", None),
    ("I think the plan is fine.", None),
    ("The meeting went well.", None),
    ("Tendai is a good friend.", None),
    ("I love this recipe.", None),
    ("Weather was lovely today.", None),
    ("What do we need to buy?", None),
    ("This needs to be reviewed by legal.", None),
    ("It will rain tomorrow.", None),
    ("You will love the new office.", None),
    ("They have to leave at six.", None),
    ("We must be careful with the budget.", None),
    ("The todo list is long.", None),
    ("Rudo will be late.", None),
    ("Tendai to the rescue!", None),
    ("Ask me anything.", None),
    ("Check the lease.", None),
    ("Checked whether the invoice was paid.", None),
    ("Found out the shop closes at five.", None),
    ("Remembered to lock the door.", None),
    ("Following up with Rudo went well.", None),
    ("I will be there.", None),
    ("My plan is simple.", None),
    ("Need.", None),
    ("I'll", None),
    ("TODO", None),
    ("The IDEA to send was good.", None),
    ("NASA will send a probe.", None),
    ("Do not forget the good times.", None),
    ("Reminders are in Settings.", None),
    ("A note about the deck.", None),
    ("Pending approval.", None),
    ("The deck is ready for Tendai.", None),
    ("Notes from the call with Anesu.", None),
    ("Everything worked.", None),
    ("Todo lists help me focus.", None),
    ("I have too many tabs open.", None),
    ("I'm tired.", None),
    ("The quote from Rudo arrived.", None),
    ("Don't worry about it.", None),
    ("Don't panic.", None),
    ("To be continued.", None),
    ("Action movies are fun.", None),
    ("Ask Questions Later.", None),
    ("Find out", None),
    ("Wait for it.", None),
    ("Wait.", None),
    ("Sunday was quiet.", None),
    ("We went to the market and bought mangoes.", None),
    ("He will be fine.", None),
    ("She wants to send flowers.", None),
    ("It was great seeing everyone again.", None),
];

fn found(sentence: &str) -> Option<LoopKind> {
    loop_items(sentence).first().map(|l| l.kind)
}

#[test]
fn the_corpus_has_enough_labelled_sentences() {
    assert!(CORPUS.len() >= 150, "{}", CORPUS.len());
}

#[test]
fn the_corpus_precision_is_at_least_95_percent() {
    let mut true_pos = 0;
    let mut false_pos = Vec::new();
    let mut missed = Vec::new();
    for (sentence, want) in CORPUS {
        match (want, found(sentence)) {
            (Some(w), Some(g)) if *w == g => true_pos += 1,
            (Some(_), _) => missed.push(*sentence),
            (None, Some(_)) => false_pos.push(*sentence),
            (None, None) => {}
        }
    }
    let precision = true_pos as f64 / (true_pos + false_pos.len()) as f64;
    assert!(
        precision >= 0.95,
        "precision {precision:.3}; false positives: {false_pos:?}"
    );
    let positives = CORPUS.iter().filter(|(_, w)| w.is_some()).count();
    assert!(
        true_pos as f64 / positives as f64 >= 0.9,
        "missed: {missed:?}"
    );
}

#[test]
fn every_loop_in_the_corpus_has_its_phrase_in_range() {
    for (sentence, _) in CORPUS {
        for item in loop_items(sentence) {
            let (a, b) = item.span;
            assert!(a <= b && b <= item.text.len(), "{sentence}");
            assert!(item.text.is_char_boundary(b));
            assert!(item.text.chars().count() <= MAX_TEXT_CHARS);
        }
    }
}

#[test]
fn the_phrase_span_covers_what_matched() {
    let item = &loop_items("I'll send Tendai the deck.")[0];
    assert_eq!(&item.text[item.span.0..item.span.1], "I'll");
    let item = &loop_items("waiting on Rudo for the quote")[0];
    assert_eq!(&item.text[item.span.0..item.span.1], "waiting on");
    let item = &loop_items("- and also need to check the lease")[0];
    assert_eq!(item.text, "need to check the lease");
}

#[test]
fn due_dates_come_from_the_sentence() {
    let item = &loop_items("I need to renew the visa @2026-12-01.")[0];
    assert_eq!(item.due.as_deref(), Some("2026-12-01"));
}

#[test]
fn tasks_headings_quotes_fences_and_front_matter_are_not_loops() {
    let body = "---\ntitle: x\nI'll hide in front matter\n---\n# I'll be a heading\n\
                - [ ] I'll be a task\n- [x] need to be a done task\n> I'll be quoted\n\
                ```\nI'll be code\n```\n    I'll be indented code\nI'll call real\n";
    let items = loop_items(body);
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0].text, "I'll call real");
    assert_eq!(items[0].line, 12);
}

#[test]
fn several_sentences_on_a_line_each_count() {
    let items = loop_items("Fine day. I'll call Rudo. Need to pay rent! Done.");
    let texts: Vec<_> = items.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(texts, ["I'll call Rudo.", "Need to pay rent!"]);
}

#[test]
fn list_markers_are_looked_through() {
    for line in [
        "- I'll call Rudo",
        "* I'll call Rudo",
        "1. I'll call Rudo",
        "  - I'll call Rudo",
    ] {
        assert_eq!(loop_items(line).len(), 1, "{line}");
    }
}

#[test]
fn long_sentences_are_cut() {
    let line = format!("I'll {}", "word ".repeat(100));
    assert!(loop_items(&line)[0].text.chars().count() <= MAX_TEXT_CHARS);
}

#[test]
fn keys_ignore_case_spacing_and_line() {
    let a = loop_key("note:x", "I'll  send the deck.");
    assert_eq!(a, loop_key("note:x", "i'll send the deck"));
    assert_ne!(a, loop_key("note:y", "I'll send the deck."));
    assert_ne!(a, loop_key("note:x", "I'll send the budget."));
    let moved = "intro\n\nmore\nI'll send the deck.";
    let k1 = loop_key("note:x", &loop_items("I'll send the deck.")[0].text);
    let k2 = loop_key("note:x", &loop_items(moved)[0].text);
    assert_eq!(k1, k2);
}

#[test]
fn the_field_round_trips_and_ignores_garbage() {
    let mut loops = Loops::default();
    loops.closed.insert("a".into(), 5);
    loops.dismissed.insert("b".into(), 6);
    loops.ai.insert(
        "c".into(),
        AiLoop {
            key: "c".into(),
            text: "t".into(),
            kind: LoopKind::Waiting,
            approved_at: 7,
        },
    );
    let mut fields = Map::new();
    fields.insert(LOOPS_FIELD.into(), loops_value(&loops));
    assert_eq!(read_loops(&fields), loops);
    fields.insert(
        LOOPS_FIELD.into(),
        json!({"closed": {"a": "x"}, "ai": [{"key": 1}]}),
    );
    assert!(read_loops(&fields).is_empty());
    fields.insert(LOOPS_FIELD.into(), json!("nope"));
    assert!(read_loops(&fields).is_empty());
}

#[test]
fn hiding_prunes_dead_keys_and_keeps_the_new_one() {
    let mut loops = Loops::default();
    loops.closed.insert("old".into(), 1);
    loops.closed.insert("live".into(), 2);
    let next = hide(&loops, "new", true, 9, &["live".to_owned()]);
    assert_eq!(next.closed.keys().collect::<Vec<_>>(), ["live"]);
    assert_eq!(next.dismissed.get("new"), Some(&9));
}

#[test]
fn locate_finds_the_nearest_line() {
    let body = "I'll send the deck.\nmid\n\nI'll   send the DECK.\n- [ ] I'll send the deck.";
    assert_eq!(locate(body, "I'll send the deck.", None), Some(0));
    assert_eq!(locate(body, "I'll send the deck.", Some(3)), Some(3));
    assert_eq!(locate(body, "never written", None), None);
}

#[test]
fn ages_read_naturally() {
    let day = 86_400_000;
    assert_eq!(age_label(0, 0), "today");
    assert_eq!(age_label(0, day), "yesterday");
    assert_eq!(age_label(0, 3 * day), "3 days ago");
    assert_eq!(age_label(0, 21 * day), "3 weeks ago");
    assert_eq!(age_label(0, 90 * day), "3 months ago");
}

fn loops_strategy() -> impl Strategy<Value = Loops> {
    let times = prop::collection::btree_map("[a-d]", 0i64..100, 0..5);
    let ai = prop::collection::vec(("[a-d]", "[a-c]{1,3}", 0i64..50, 0usize..3), 0..4).prop_map(
        |list| {
            list.into_iter()
                .map(|(key, text, approved_at, kind)| {
                    let kind = [LoopKind::Mine, LoopKind::Waiting, LoopKind::Question][kind];
                    (
                        key.clone(),
                        AiLoop {
                            key,
                            text,
                            kind,
                            approved_at,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>()
        },
    );
    (times.clone(), times, ai).prop_map(|(closed, dismissed, ai)| Loops {
        closed,
        dismissed,
        ai,
    })
}

fn body_strategy() -> impl Strategy<Value = String> {
    let line = prop_oneof![
        Just("I'll call Rudo.".to_owned()),
        Just("- [ ] a task".to_owned()),
        Just("```".to_owned()),
        Just("~~~".to_owned()),
        Just("> quote".to_owned()),
        Just("Waiting on Tendai".to_owned()),
        Just("Need to  check the lease?".to_owned()),
        Just("---".to_owned()),
        Just("# head".to_owned()),
        "\\PC{0,40}",
    ];
    prop::collection::vec(line, 0..14).prop_map(|lines| lines.join("\n"))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn prop_loop_items_never_panic_and_lines_are_in_range(body in "\\PC{0,300}") {
        let lines = body.lines().count() as u32;
        for item in loop_items(&body) {
            prop_assert!(item.line < lines.max(1));
            prop_assert!(item.span.0 <= item.span.1 && item.span.1 <= item.text.len());
            prop_assert!(item.text.is_char_boundary(item.span.1));
            prop_assert!(item.text.chars().count() <= MAX_TEXT_CHARS);
            prop_assert!(!item.text.trim().is_empty());
        }
    }

    #[test]
    fn prop_loop_items_survive_structured_bodies(body in body_strategy()) {
        let lines = body.lines().count();
        for item in loop_items(&body) {
            prop_assert!((item.line as usize) < lines);
        }
    }

    /// Nothing inside a code fence or on a task line is ever a loop.
    #[test]
    fn prop_no_loop_inside_code_fences_or_tasks(
        before in prop::collection::vec("[a-z ]{0,12}", 0..3),
        inner in prop::collection::vec(prop_oneof![
            Just("I'll call Rudo.".to_owned()),
            Just("Waiting on Tendai".to_owned()),
            Just("TODO: x y z".to_owned()),
        ], 1..4),
        fence in prop_oneof![Just("```"), Just("~~~")],
        checked in any::<bool>(),
    ) {
        let mut body = before.join("\n");
        if !body.is_empty() {
            body.push('\n');
        }
        let start = body.lines().count() as u32;
        body.push_str(fence);
        body.push('\n');
        for line in &inner {
            body.push_str(line);
            body.push('\n');
        }
        body.push_str(fence);
        body.push('\n');
        let task_line = body.lines().count() as u32;
        body.push_str(if checked { "- [x] I'll call Rudo.\n" } else { "- [ ] I'll call Rudo.\n" });
        for item in loop_items(&body) {
            let inside = item.line > start && item.line <= start + inner.len() as u32;
            prop_assert!(!inside, "loop in fence at {}", item.line);
            prop_assert!(item.line != task_line, "loop on task line");
        }
    }

    #[test]
    fn prop_merge_is_commutative_associative_and_idempotent(
        a in loops_strategy(), b in loops_strategy(), c in loops_strategy()
    ) {
        let ab = merge_loops(&a, &b);
        prop_assert_eq!(&ab, &merge_loops(&b, &a));
        prop_assert_eq!(merge_loops(&ab, &c), merge_loops(&a, &merge_loops(&b, &c)));
        prop_assert_eq!(merge_loops(&a, &a), a.clone());
        let mut fields = Map::new();
        fields.insert(LOOPS_FIELD.into(), loops_value(&ab));
        prop_assert_eq!(read_loops(&fields), ab);
    }

    /// A loop's key doesn't depend on where it is: padding the body with
    /// lines above it leaves the keys alone.
    #[test]
    fn prop_loop_key_is_stable_across_line_moves(pad in 0usize..6) {
        let base = "I'll call Rudo.\nWaiting on Tendai for the quote";
        let moved = format!("{}{base}", "filler\n".repeat(pad));
        let keys = |b: &str| -> Vec<String> {
            loop_items(b).iter().map(|l| loop_key("n", &l.text)).collect()
        };
        prop_assert_eq!(keys(base), keys(&moved));
    }
}
