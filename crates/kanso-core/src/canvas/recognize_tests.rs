use proptest::prelude::*;

use super::tests::config;
use super::*;

fn stroke(id: &str, seed: u32) -> Stroke {
    Stroke {
        id: id.into(),
        tool: "pen".into(),
        color: "#1e1e1e".into(),
        width: 3.0,
        points: (0..6)
            .map(|i| Point::new(f64::from(i * 7 + seed), f64::from(i * i + seed), 0.5))
            .collect(),
        ..Stroke::default()
    }
}

/// `n` ink layers `i0..`, each with one stroke.
fn board(n: usize) -> Canvas {
    let mut c = Canvas::default();
    for i in 0..n {
        c.put_node(
            Node {
                id: format!("i{i}"),
                kind: INK.into(),
                x: i as f64 * 400.0,
                ..Node::default()
            },
            1,
        );
        assert!(c.add_stroke(&format!("i{i}"), stroke(&format!("s{i}"), i as u32), 2));
    }
    c
}

/// Reads every waiting node, the way a pass over a canvas does.
fn read_all(c: &mut Canvas) -> Vec<String> {
    let waiting = c.ink_to_read();
    let mut seen = Vec::new();
    for job in waiting {
        let text = format!("words of {}", job.node);
        assert!(c.keep_ink_text(&job.node, &job.md5, "fake", &text));
        seen.push(job.node);
    }
    seen
}

#[test]
fn highlighter_and_empty_ink_have_nothing_to_read() {
    let mut c = board(1);
    assert_eq!(c.ink_to_read().len(), 1);
    let mut wash = stroke("h", 3);
    wash.tool = "highlighter".into();
    let mut only = Canvas::default();
    only.put_node(
        Node {
            id: "i".into(),
            kind: INK.into(),
            ..Node::default()
        },
        1,
    );
    assert!(only.ink_to_read().is_empty());
    only.add_stroke("i", wash, 2);
    assert!(only.ink_to_read().is_empty());
    // Erasing the writing leaves its words with nothing to stand on.
    read_all(&mut c);
    assert!(c.remove_stroke("i0", "s0", 3));
    assert!(c.ink_to_read().is_empty());
    assert!(c.prune_ink_text());
    assert!(c.search_text().is_empty());
}

#[test]
fn words_read_from_old_strokes_are_refused() {
    let mut c = board(1);
    let job = c.ink_to_read().remove(0);
    c.add_stroke("i0", stroke("s9", 9), 3);
    assert!(!c.keep_ink_text("i0", &job.md5, "fake", "stale"));
    assert!(c.kanso_ink_ocr.is_empty());
    assert!(!c.keep_ink_text("nope", &job.md5, "fake", "x"));
}

proptest! {
    #![proptest_config(config(48))]

    /// Moving a layer or recolouring a stroke does not change what is
    /// read; drawing does.
    #[test]
    fn prop_fingerprint_follows_the_handwriting_only(
        dx in -500.0f64..500.0, dy in -500.0f64..500.0, seed in 0u32..50,
    ) {
        let mut c = board(1);
        let before = c.nodes[0].ink_fingerprint().unwrap();
        c.nodes[0].x += dx;
        c.nodes[0].y += dy;
        c.nodes[0].strokes[0].color = "#e03131".into();
        c.nodes[0].strokes[0].kanso_version += 1;
        prop_assert_eq!(c.nodes[0].ink_fingerprint().unwrap(), before.clone());
        c.add_stroke("i0", stroke("extra", seed), 5);
        prop_assert_ne!(c.nodes[0].ink_fingerprint().unwrap(), before);
    }

    /// After a full read, only the layers whose strokes changed are
    /// offered, and reading them leaves every other layer's entry as it
    /// was.
    #[test]
    fn prop_incremental_reread_touches_only_changed_nodes(
        changed in proptest::collection::vec(any::<bool>(), 1..7),
        seed in 0u32..50,
    ) {
        let mut c = board(changed.len());
        prop_assert_eq!(read_all(&mut c).len(), changed.len());
        prop_assert!(c.ink_to_read().is_empty());
        let before = c.kanso_ink_ocr.clone();
        let mut want = Vec::new();
        for (i, &edit) in changed.iter().enumerate() {
            if edit {
                let id = format!("i{i}");
                let added = c.add_stroke(&id, stroke(&format!("n{i}"), seed), 9);
                prop_assert!(added);
                want.push(id);
            }
        }
        let offered: Vec<String> = c.ink_to_read().into_iter().map(|j| j.node).collect();
        prop_assert_eq!(&offered, &want);
        let read = read_all(&mut c);
        prop_assert_eq!(&read, &want);
        for old in &before {
            if !want.contains(&old.node) {
                prop_assert!(c.kanso_ink_ocr.contains(old), "{} changed", old.node);
            }
        }
        prop_assert!(c.ink_to_read().is_empty());
        prop_assert_eq!(c.kanso_ink_ocr.len(), changed.len());
    }

    /// What was recognized is what search reads, beside the cards, and
    /// the JSON is not.
    #[test]
    fn prop_search_text_includes_recognized_words(
        words in proptest::collection::vec("[a-z]{4,9}", 1..5),
    ) {
        let mut c = board(words.len());
        c.put_node(
            Node {
                id: "t".into(),
                kind: "text".into(),
                text: Some("card words".into()),
                width: 10.0,
                height: 10.0,
                ..Node::default()
            },
            3,
        );
        for (i, w) in words.iter().enumerate() {
            let job = c.ink_to_read().into_iter().find(|j| j.node == format!("i{i}")).unwrap();
            prop_assert!(c.keep_ink_text(&job.node, &job.md5, "fake", w));
        }
        let text = c.search_text();
        for w in &words {
            prop_assert!(text.contains(w.as_str()));
        }
        prop_assert!(text.contains("card words"));
        prop_assert!(!text.contains("strokes") && !text.contains("kansoVersion"));
    }
}
