//! The recording player card in a preview, headless (issue #38, steps 5 and
//! 6): it draws over a recording's audio line, says when the bytes aren't
//! here, follows timestamp links, and takes the keys. The sound card itself
//! is not needed: the engine has its own tests in `kanso-audio`.

use std::sync::Arc;

use gpui_kit::{AppContext as _, Entity, IntoElement, Render, TestAppContext, VisualTestContext};
use kanso_audio::{Channels, RecordSource, Recorder, RecorderOptions};
use kanso_engine::recording_flow::{Destination, Recorded, save_recording};
use kanso_engine::recording_player::AudioMoment;
use kanso_store::{Origin, Store};

use crate::player::RecordingPlayer;
use crate::player_model::{Phase, PlayerEvent};
use crate::preview::Preview;

struct Fixture {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: gpui_kit::AnyWindowHandle,
    preview: Entity<Preview>,
    /// The note and its first part.
    note_id: String,
    file_ids: Vec<String>,
    body: String,
}

/// Seconds of a steady tone in as many parts as asked for.
fn record(dir: &std::path::Path, seconds: usize) -> Vec<std::path::PathBuf> {
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
    options.id = Some("play-1".into());
    let mut recorder = Recorder::create(&dir.join("in-progress"), options).unwrap();
    let tone: Vec<f32> = (0..48_000 * seconds)
        .map(|i| 0.4 * (std::f32::consts::TAU * 330.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&tone).unwrap();
    recorder
        .finish()
        .unwrap()
        .parts
        .into_iter()
        .map(|(path, _)| path)
        .collect()
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let parts = record(dir.path(), 5);
    let saved = save_recording(
        &store,
        &Recorded {
            session_id: "play-1".into(),
            parts,
            duration_secs: 5,
            mic_only: true,
            when: "4 Nov 2026, 14:05".into(),
        },
        &Destination::NewNote { book_id: None },
        Origin::Ui,
    )
    .unwrap();
    let body = store.note(&saved.note_id).unwrap().unwrap().body;
    let app_store = store.clone();
    let (window, preview) = cx.update(|cx| {
        gpui_kit::init(cx);
        let (links, _rx) = async_channel::unbounded();
        let (resizes, _resized) = async_channel::unbounded();
        let (window, preview) = gpui_kit::open_window(Default::default(), cx, |_, cx| {
            cx.new(|cx| Preview::new(app_store, links, resizes, cx))
        })
        .unwrap();
        (window, preview)
    });
    let fixture = Fixture {
        _dir: dir,
        store,
        window,
        preview,
        note_id: saved.note_id,
        file_ids: saved.file_ids,
        body,
    };
    show(
        cx,
        &fixture,
        &fixture.note_id.clone(),
        &fixture.body.clone(),
    );
    fixture
}

/// Shows `body` as note `note_id`'s preview, drawn once.
fn show(cx: &mut TestAppContext, f: &Fixture, note_id: &str, body: &str) {
    let (id, text) = (note_id.to_owned(), body.to_owned());
    f.preview.update(cx, |preview, cx| {
        preview.set_note(Some(id), cx);
        preview.set_text(text.into(), cx);
    });
    cx.run_until_parked();
    draw(cx, f);
}

fn draw(cx: &mut TestAppContext, f: &Fixture) {
    f.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    cx.run_until_parked();
}

fn player(f: &Fixture, cx: &mut TestAppContext) -> Entity<RecordingPlayer> {
    cx.read(|cx| f.preview.read(cx).player().clone())
}

fn bounds(
    f: &Fixture,
    cx: &mut TestAppContext,
    selector: &'static str,
) -> Option<gpui_kit::Bounds<gpui_kit::Pixels>> {
    let mut vcx = VisualTestContext::from_window(f.window, cx);
    vcx.debug_bounds(selector)
}

#[gpui_kit::test]
fn the_audio_line_draws_a_player_card_with_its_controls(cx: &mut TestAppContext) {
    let f = setup(cx);
    assert!(f.body.starts_with("[Audio · "), "{}", f.body);
    for selector in [
        "recording-player",
        "player-toggle",
        "player-scrub",
        "player-elapsed",
        "player-total",
        "player-speed",
        "player-mute",
        "player-volume",
    ] {
        assert!(bounds(&f, cx, selector).is_some(), "{selector} is drawn");
    }
    assert!(bounds(&f, cx, "player-unavailable").is_none());
    // The length was read from the files in the background.
    let p = player(&f, cx);
    assert_eq!(cx.read(|cx| p.read(cx).length_secs()), Some(5));
    assert_eq!(cx.read(|cx| p.read(cx).phase().clone()), Phase::Idle);
}

#[gpui_kit::test]
fn a_note_without_a_recording_has_no_card(cx: &mut TestAppContext) {
    let f = setup(cx);
    show(cx, &f, "note:other", "# Just words\n\nNo audio here.");
    assert!(bounds(&f, cx, "recording-player").is_none_or(|b| b.size.height <= gpui_kit::px(1.)));
    assert!(bounds(&f, cx, "player-toggle").is_none());
}

#[gpui_kit::test]
fn bytes_that_have_not_arrived_say_so_and_offer_sync_now(cx: &mut TestAppContext) {
    let f = setup(cx);
    // A file the account has told us about, whose bytes are not here yet.
    f.store
        .apply_remote(
            &kanso_store::SyncDoc {
                id: "file:later-1".into(),
                rev: "1-aa".into(),
                deleted: false,
                fields: serde_json::json!({
                    "name": "Recording part 1.opus", "contentType": "audio/ogg",
                    "md5digest": "00000000000000000000000000000000", "createdAt": 1,
                })
                .as_object()
                .unwrap()
                .clone(),
                base_rev: None,
            },
            None,
            None,
        )
        .unwrap();
    show(
        cx,
        &f,
        "note:later",
        "[Audio · 3 min](inkdrop://file:later-1)\n",
    );
    assert!(bounds(&f, cx, "player-unavailable").is_some());
    assert!(bounds(&f, cx, "player-sync-now").is_some());
    assert!(bounds(&f, cx, "player-toggle").is_none());
    let p = player(&f, cx);
    assert_eq!(cx.read(|cx| p.read(cx).phase().clone()), Phase::NotHere);
}

#[gpui_kit::test]
fn a_recording_whose_file_is_gone_says_so(cx: &mut TestAppContext) {
    let f = setup(cx);
    show(
        cx,
        &f,
        "note:gone",
        "[Audio · 3 min](inkdrop://file:ghost-1)\n",
    );
    assert!(bounds(&f, cx, "player-unavailable").is_some());
    assert!(bounds(&f, cx, "player-sync-now").is_none());
    let p = player(&f, cx);
    assert_eq!(cx.read(|cx| p.read(cx).phase().clone()), Phase::Missing);
}

#[gpui_kit::test]
fn a_timestamp_link_brings_the_player_up_and_plays_that_moment(cx: &mut TestAppContext) {
    let f = setup(cx);
    let p = player(&f, cx);
    p.update(cx, |p, _| p.silence());
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = events.clone();
    let _sub = cx.update(|cx| {
        cx.subscribe(&p, move |_, event: &PlayerEvent, _| {
            seen.lock().unwrap().push(*event)
        })
    });
    let moment = AudioMoment {
        file_id: f.file_ids[0].clone(),
        seconds: 3,
    };
    let known = p.update(cx, |p, cx| p.play_moment(&moment, cx));
    assert!(known);
    assert_eq!(*events.lock().unwrap(), vec![PlayerEvent::Moment]);
    // A moment of a recording this note doesn't have is not followed.
    let foreign = AudioMoment {
        file_id: "file:elsewhere".into(),
        seconds: 3,
    };
    assert!(!p.update(cx, |p, cx| p.play_moment(&foreign, cx)));
    // The audio opens in the background: on a machine with a speaker it
    // ends up ready at 3 s and playing; on one without, it says why.
    for _ in 0..200 {
        cx.run_until_parked();
        let phase = cx.read(|cx| p.read(cx).phase().clone());
        if !matches!(phase, Phase::Opening) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    match cx.read(|cx| p.read(cx).phase().clone()) {
        Phase::Ready => {
            let at = cx.read(|cx| p.read(cx).position_us());
            assert!((3_000_000..5_000_000).contains(&at), "at {at} us");
        }
        Phase::Failed(why) => assert!(!why.is_empty()),
        other => panic!("unexpected {other:?}"),
    }
    draw(cx, &f);
    assert!(bounds(&f, cx, "recording-player").is_some());
}

#[gpui_kit::test]
fn changing_note_stops_and_forgets_the_player(cx: &mut TestAppContext) {
    let f = setup(cx);
    let p = player(&f, cx);
    p.update(cx, |p, _| p.silence());
    let moment = AudioMoment {
        file_id: f.file_ids[0].clone(),
        seconds: 1,
    };
    p.update(cx, |p, cx| p.play_moment(&moment, cx));
    show(cx, &f, "note:next", "Nothing to hear.");
    assert!(!cx.read(|cx| p.read(cx).is_open()));
    assert_eq!(cx.read(|cx| p.read(cx).phase().clone()), Phase::Idle);
    assert!(!cx.read(|cx| p.read(cx).is_playing()));
    assert!(bounds(&f, cx, "player-toggle").is_none());
}

#[gpui_kit::test]
fn space_and_the_arrows_are_the_players_when_it_has_the_keyboard(cx: &mut TestAppContext) {
    use gpui_kit::Keystroke;
    let f = setup(cx);
    let p = player(&f, cx);
    p.update(cx, |p, _| p.silence());
    let key = |text: &str, cx: &mut TestAppContext| {
        let ks = Keystroke::parse(text).unwrap();
        p.update(cx, |p, cx| p.key_for_test(&ks, cx))
    };
    assert!(key("space", cx));
    assert!(key("left", cx));
    assert!(key("right", cx));
    assert!(key("shift-left", cx));
    assert!(key("shift-right", cx));
    // Not the player's: letters, and chords with the command key.
    assert!(!key("a", cx));
    assert!(!key("cmd-left", cx));
    assert!(!key("shift-space", cx));
}

/// Drawing the card must never read the card's own entity again, which GPUI
/// refuses with a panic ("cannot read RecordingPlayer while it is already
/// being updated"): 0.9.4 crashed on that in another view.
// no-prop-test: drives one view's render.
#[gpui_kit::test]
fn the_card_draws_without_reading_itself(cx: &mut TestAppContext) {
    let f = setup(cx);
    let p = player(&f, cx);
    f.window
        .update(cx, |_, window, cx| {
            p.update(cx, |card, cx| {
                let _ = card.render(window, cx).into_any_element();
            });
        })
        .unwrap();
}
