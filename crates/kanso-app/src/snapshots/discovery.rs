//! Snapshot states for discovery: the inspector's graph and related notes,
//! the library's graph, Settings' related notes in each state, and the
//! palette finding notes by meaning.

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_core::DocType;
use kanso_store::{Origin, Put, Store};
use serde_json::json;

use super::{BOTTOM_BAND_HEIGHT, DAY, DEMO_NOW_MS, MINUTE, Shot, assert_surface, shot_colors};
use crate::config::InspectorTab;
use crate::layout::TOP_BAND_HEIGHT;
use crate::related::{MODEL_ID, Related, Status};
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

/// Notes around the demo's "MCP support model" that link to it and to
/// each other with wiki-links, so the graphs have something to draw.
fn link_cluster(store: &Store) {
    let book = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Engineering")
        .map(|b| b.id)
        .unwrap();
    let research = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Research")
        .map(|b| b.id)
        .unwrap();
    let notes = [
        (
            &book,
            "Permission manager",
            "Grants for each client, from the [[MCP support model]]. Tools come from the [[Tool registry schema]].",
        ),
        (
            &book,
            "Activity log",
            "Every write by a client is recorded, as the [[Permission manager]] asks.",
        ),
        (
            &book,
            "Client approvals",
            "A new client waits for approval in the [[Permission manager]]; each approval lands in the [[Activity log]].",
        ),
        (
            &research,
            "Offline mode",
            "Agents keep working against the local store; see [[Sync design]] and the [[MCP support model]].",
        ),
        (
            &research,
            "Local-first notes",
            "Why the library lives on the device first. Related: [[Offline mode]], [[Kanso roadmap]].",
        ),
        (
            &book,
            "Q3 planning",
            "Themes for the quarter: the [[Kanso roadmap]] and the [[Release checklist]].",
        ),
    ];
    for (ix, (book, title, body)) in notes.into_iter().enumerate() {
        let at = DEMO_NOW_MS - 5 * DAY - ix as i64 * 7 * MINUTE;
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book,
                    "createdAt": at - DAY, "updatedAt": at}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
}

fn show_tab(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>, tab: InspectorTab) {
    cx.update(|cx| workspace.update(cx, |workspace, cx| workspace.show_inspector(Some(tab), cx)));
}

/// The open note's graph, one link out.
pub(super) fn inspector_graph(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    link_cluster(store);
    show_tab(cx, workspace, InspectorTab::Graph);
}

/// The open note's graph, two links out.
pub(super) fn inspector_graph_two(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    inspector_graph(cx, window, workspace, store);
    cx.update(|cx| {
        let graph = workspace.read(cx).inspector.read(cx).graph.clone();
        graph.update(cx, |graph, cx| graph.set_hops(2, cx));
    });
}

fn open_graph(cx: &mut HeadlessAppContext, window: AnyWindowHandle, workspace: &Entity<Workspace>) {
    cx.update_window(window, |_, window, cx| {
        workspace.update(cx, |workspace, cx| workspace.show_graph(window, cx));
    })
    .unwrap();
}

/// The library's graph over the window.
pub(super) fn library_graph(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    link_cluster(store);
    open_graph(cx, window, workspace);
}

/// The library's graph with the pointer on a note: it and its neighbours
/// lit, the rest dimmed.
pub(super) fn library_graph_hover(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    library_graph(cx, window, workspace, store);
    cx.run_until_parked();
    cx.update(|cx| {
        let graph = workspace.read(cx).graph.clone().unwrap();
        graph.update(cx, |graph, cx| {
            graph.hover_for_test("Permission manager", cx)
        });
    });
}

/// The library's graph narrowed to one notebook.
pub(super) fn library_graph_notebook(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    link_cluster(store);
    let engineering = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == "Engineering")
        .unwrap()
        .id;
    open_graph(cx, window, workspace);
    cx.update(|cx| {
        let graph = workspace.read(cx).graph.clone().unwrap();
        graph.update(cx, |graph, cx| {
            graph.set_filter(crate::graph_view::GraphFilter::Book(engineering), cx)
        });
    });
}

/// A library of 2,000 notes in topic clusters, each note linking to a
/// few others: the graph culled and with fewer labels as it zooms out.
pub(super) fn library_graph_large(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    let books: Vec<String> = store.books().unwrap().into_iter().map(|b| b.id).collect();
    let topics = [
        "Sync", "Agents", "Design", "Release", "Reading", "Research", "Platform", "Notes",
    ];
    let n = 2_000usize;
    let ids: Vec<String> = (0..n).map(|i| format!("note:gen{i:05}")).collect();
    let mut rng = 0x2545_F491_4F6C_DD1Du64;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    for i in 0..n {
        let topic = i % topics.len();
        let mut body = String::new();
        for _ in 0..(1 + next() % 3) {
            // Mostly within the topic, sometimes across.
            let j = if next() % 5 == 0 {
                (next() % n as u64) as usize
            } else {
                (topic + topics.len() * (next() % (n / topics.len()) as u64) as usize) % n
            };
            if j != i {
                body.push_str(&format!(
                    "See [x](inkdrop://note/{}).\n",
                    ids[j].strip_prefix("note:").unwrap()
                ));
            }
        }
        store
            .put(
                DocType::Note,
                json!({"_id": ids[i], "title": format!("{} note {i}", topics[topic]),
                    "body": body, "bookId": books[topic % books.len()],
                    "createdAt": DEMO_NOW_MS - 400 * DAY + i as i64, "updatedAt": DEMO_NOW_MS - 300 * DAY}),
                Put::Upsert,
                Origin::Import,
            )
            .unwrap();
    }
    let started = std::time::Instant::now();
    let (notes, links) = store.graph_rows(true).unwrap();
    let graph = crate::graph::Graph::build(notes, &links);
    crate::graph::layout(&graph, 1);
    println!(
        "graph of {} notes and {} links laid out in {:?} (debug build)",
        graph.len(),
        graph.link_count(),
        started.elapsed()
    );
    open_graph(cx, window, workspace);
}

/// A vector for `text` from its longer words, hashed into 48 numbers:
/// notes that share words point the same way.
fn fake_vector(text: &str) -> Vec<f32> {
    let mut v = vec![0f32; 48];
    for word in text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4)
    {
        let hash = word
            .to_lowercase()
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
                (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
            });
        v[(hash % 48) as usize] += 1.;
    }
    v
}

fn set_related(cx: &mut HeadlessAppContext, status: Status, on: bool) {
    cx.update(|cx| {
        cx.global_mut::<AppState>().config.discovery.related_notes = on;
        let related = cx.default_global::<Related>();
        related.status = status;
        related.generation += 1;
    });
}

/// Embeds every note with [`fake_vector`] under the app's model name.
fn fake_embeddings(store: &Store) -> usize {
    store.set_embedding_model(Some(MODEL_ID)).unwrap();
    let queue = store.embedding_queue(MODEL_ID).unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|p| {
            let v = fake_vector(&p.text);
            (p.id, p.hash, v)
        })
        .collect();
    store.put_embeddings(MODEL_ID, &rows).unwrap();
    queue.total
}

/// The Related tab, with every note embedded.
pub(super) fn inspector_related(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    link_cluster(store);
    let notes = fake_embeddings(store);
    set_related(cx, Status::Ready { notes }, true);
    show_tab(cx, workspace, InspectorTab::Related);
}

/// [`fake_vector`] as a model, for searches by meaning.
struct FakeModel;

impl kanso_store::Embedder for FakeModel {
    fn model(&self) -> &str {
        MODEL_ID
    }
    fn dim(&self) -> usize {
        48
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts.iter().map(|t| fake_vector(t)).collect()
    }
}

/// ⌘P with words no note has together: the notes about them, found by
/// meaning, the closest selected.
pub(super) fn palette_meaning(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    link_cluster(store);
    let notes = fake_embeddings(store);
    store.lend_embedder(Some(std::sync::Arc::new(FakeModel)));
    set_related(cx, Status::Ready { notes }, true);
    super::open_palette(
        cx,
        window,
        workspace,
        crate::palette::Mode::Notes,
        "client approval permission grants",
    );
    cx.advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    cx.update(|cx| {
        let palette = workspace.read(cx).palette.clone();
        palette.update(cx, |palette, cx| palette.select_first_note(cx));
    });
}

/// Tags close notes have, as the open note's suggestions.
fn suggest_tags(cx: &mut HeadlessAppContext, workspace: &Entity<Workspace>, store: &Store) {
    let suggested: Vec<kanso_store::TagSuggestion> = [("roadmap", 1.4), ("rust", 0.6)]
        .into_iter()
        .map(|(name, weight)| kanso_store::TagSuggestion {
            tag_id: store.tag_named(name, Origin::Ui).unwrap(),
            name: name.into(),
            weight,
        })
        .collect();
    cx.update(|cx| {
        let meaning = workspace.read(cx).editor.read(cx).meaning().clone();
        meaning.update(cx, |meaning, cx| {
            meaning.set_suggested_for_test(suggested);
            cx.notify();
        });
    });
}

/// The Related tab with tags to suggest above the related notes.
pub(super) fn inspector_related_tags(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    inspector_related(cx, window, workspace, store);
    suggest_tags(cx, workspace, store);
}

/// The tag list from the header, the suggested tags first.
pub(super) fn palette_suggested_tags(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    suggest_tags(cx, workspace, store);
    super::pick_from_header(cx, window, workspace, crate::commands::Picker::Tag, "");
}

/// The open note with a near twin: the quiet line under its header.
pub(super) fn editor_twin(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    cx.update(|cx| {
        let meaning = workspace.read(cx).editor.read(cx).meaning().clone();
        meaning.update(cx, |meaning, cx| {
            meaning.set_duplicate_for_test(Some(kanso_store::RelatedNote {
                id: "note:twin".into(),
                title: "MCP support model (copy)".into(),
                score: 0.97,
            }));
            cx.notify();
        });
    });
}

/// The Related tab while the feature is off.
pub(super) fn inspector_related_off(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_related(cx, Status::Off, false);
    show_tab(cx, workspace, InspectorTab::Related);
}

/// The Related tab while the model downloads.
pub(super) fn inspector_related_downloading(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_related(
        cx,
        Status::Downloading {
            done: 11_800_000,
            total: 30_921_000,
        },
        true,
    );
    show_tab(cx, workspace, InspectorTab::Related);
}

/// The Related tab after the download failed.
pub(super) fn inspector_related_failed(
    cx: &mut HeadlessAppContext,
    _: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    set_related(
        cx,
        Status::Failed("Couldn't reach the download. Check the connection.".into()),
        true,
    );
    show_tab(cx, workspace, InspectorTab::Related);
}

fn settings_related(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    status: Status,
) {
    let on = status != Status::Off;
    set_related(cx, status, on);
    cx.update(|cx| cx.global_mut::<AppState>().config.discovery.image_text = true);
    super::show_settings(cx, window, workspace, SettingsPage::General);
    // Discovery is the General page's last section.
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace
                .settings
                .update(cx, |settings, _| settings.scroll_to_end_for_test());
        });
    });
}

pub(super) fn settings_related_off(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_related(cx, window, workspace, Status::Off);
}

pub(super) fn settings_related_downloading(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_related(
        cx,
        window,
        workspace,
        Status::Downloading {
            done: 11_800_000,
            total: 30_921_000,
        },
    );
}

pub(super) fn settings_related_indexing(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_related(
        cx,
        window,
        workspace,
        Status::Indexing {
            done: 640,
            total: 2_118,
        },
    );
}

pub(super) fn settings_related_ready(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_related(cx, window, workspace, Status::Ready { notes: 2_118 });
}

pub(super) fn settings_related_failed(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    _: &Store,
) {
    settings_related(
        cx,
        window,
        workspace,
        Status::Failed(
            "The downloaded model.safetensors isn't the expected file, so it wasn't kept.".into(),
        ),
    );
}

/// The library's graph: its bands in the list's surface, the canvas in the
/// editor's, at the corners where no note is drawn.
pub(super) fn check_graph(image: &image::RgbaImage, shot: &Shot) {
    let colors = shot_colors(shot);
    let width = f32::from(shot.size.width);
    let height = f32::from(shot.size.height);
    let scale = image.width() as f32 / width;
    for (at, color, what) in [
        ((width - 4., 4.), colors.list, "graph's top band"),
        (
            (width - 4., height - BOTTOM_BAND_HEIGHT / 2.),
            colors.list,
            "graph's bottom band",
        ),
        (
            (width - 6., TOP_BAND_HEIGHT + 6.),
            colors.editor,
            "graph's canvas",
        ),
    ] {
        assert_surface(image, scale, at, color, 2, what);
    }
}
