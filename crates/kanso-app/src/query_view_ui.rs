//! Query views drawn in the preview: a `query` fence becomes a table of the
//! notes it lists.
//!
//! Parsing and evaluation are `kanso_core::query_view`'s. The preview only
//! reads the notes (on the background executor, once for every query in the
//! note) and draws the rows; a note's change asks for them again.

use std::collections::{HashMap, HashSet};
use std::hash::{Hash as _, Hasher as _};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use gpui_kit::base::text::{MarkdownExtensions, MarkdownNode, MarkdownParseContext, markdown_ast};
use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::query_view::{self, ViewNote, ViewRow, ViewSpec};
use kanso_store::Store;

use crate::preview::LinkSender;
use crate::preview_rich::Rich;
use crate::theme::Colors;
use crate::zoom::px;

/// The custom node a query fence becomes.
pub const QUERY: &str = "kanso-query";

/// A query fence's source and the Markdown it was written as.
#[derive(Clone, Debug)]
pub struct QueryBlock {
    pub source: Arc<str>,
    pub markdown: SharedString,
}

/// The block `node` draws as, if it is a `query` fence.
pub fn parse_block(
    node: &markdown_ast::Node,
    cx: &MarkdownParseContext<'_>,
) -> Option<MarkdownNode> {
    let markdown_ast::Node::Code(code) = node else {
        return None;
    };
    if !query_view::is_query_fence(code.lang.as_deref()) {
        return None;
    }
    let source: Arc<str> = code.value.clone().into();
    let written = cx.node_source(node)?;
    Some(
        MarkdownNode::new(
            QUERY,
            QueryBlock {
                source,
                markdown: written.into(),
            },
        )
        .text(code.value.clone())
        .markdown(written),
    )
}

/// Registers the query renderer.
pub fn register(
    extensions: MarkdownExtensions,
    rich: Arc<Rich>,
    links: LinkSender,
) -> MarkdownExtensions {
    extensions.block_renderer(QUERY, move |node, window, cx| {
        render(node, &rich, &links, window, cx)
    })
}

/// Query view bookkeeping for one store. The app has one store, but tests
/// open several at once; keying by it keeps their changes apart and lets a
/// test's debounce settle.
#[derive(Default)]
struct Progress {
    /// Counts changes to the notes.
    epoch: u64,
    /// When the notes last changed, until that change settles.
    pending: Option<Instant>,
    /// The last change whose notes have been read.
    settled: u64,
}

type StoreKey = usize;
static PROGRESS: LazyLock<Mutex<HashMap<StoreKey, Progress>>> = LazyLock::new(Default::default);
/// How long a burst of saves is allowed to settle before the tables read
/// the notes again.
const SETTLE: Duration = Duration::from_millis(300);
static WAKE_SCHEDULED: LazyLock<Mutex<HashSet<StoreKey>>> = LazyLock::new(Default::default);

fn store_key(store: &Store) -> StoreKey {
    store as *const Store as usize
}

type Snapshots = HashMap<(StoreKey, bool), (u64, Arc<Vec<ViewNote>>)>;
static SNAPSHOT: LazyLock<Mutex<Snapshots>> = LazyLock::new(Default::default);
/// One read at a time: a note's queries share it.
static READING: LazyLock<Mutex<()>> = LazyLock::new(Default::default);

struct Ready {
    epoch: u64,
    rows: Arc<Vec<ViewRow>>,
}

type CacheKey = (StoreKey, u64);
type Results = HashMap<CacheKey, Ready>;
/// The last rows per (store, source, self), kept while a refresh runs.
static RESULTS: LazyLock<Mutex<Results>> = LazyLock::new(Default::default);
/// (cache key, epoch) evaluations in flight.
static WORKING: LazyLock<Mutex<HashSet<(CacheKey, u64)>>> = LazyLock::new(Default::default);

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Notes changed: query views read them again once the saves settle.
pub fn notes_changed(store: &Store) {
    let mut progress = lock(&PROGRESS);
    let state = progress.entry(store_key(store)).or_default();
    state.epoch += 1;
    state.pending = Some(Instant::now());
}

/// Settle the pending change if a quiet spell has passed.
fn settle_if_due(store: StoreKey) {
    let mut progress = lock(&PROGRESS);
    let Some(state) = progress.get_mut(&store) else {
        return;
    };
    if let Some(at) = state.pending
        && at.elapsed() >= SETTLE
    {
        state.settled = state.epoch;
        state.pending = None;
    }
}

/// The last settled change.
fn settled(store: StoreKey) -> u64 {
    lock(&PROGRESS).get(&store).map_or(0, |state| state.settled)
}

/// How long until the current change settles, if it hasn't yet.
fn pending_for(store: StoreKey) -> Option<Duration> {
    let progress = lock(&PROGRESS);
    progress
        .get(&store)
        .and_then(|state| state.pending)
        .map(|at| SETTLE.saturating_sub(at.elapsed()))
        .filter(|left| !left.is_zero())
}

/// Wake the preview when the pending change settles, so the tables read
/// the notes then.
fn wake_after(rich: &Rich, delay: Duration, store: StoreKey, cx: &mut App) {
    if !lock(&WAKE_SCHEDULED).insert(store) {
        return;
    }
    let wake = rich.wake.clone();
    let executor = cx.background_executor().clone();
    let timer = executor.clone();
    executor
        .spawn(async move {
            timer.timer(delay).await;
            wake.try_send(()).ok();
            lock(&WAKE_SCHEDULED).remove(&store);
        })
        .detach();
}

/// Every note as a query reads it, read at most once per change and kind.
/// When `need_body` is false only front matter is read.
fn notes(epoch: u64, store: &Store, need_body: bool) -> Arc<Vec<ViewNote>> {
    let snapshot_key = (store_key(store), need_body);
    if let Some((at, notes)) = lock(&SNAPSHOT).get(&snapshot_key).cloned()
        && at == epoch
    {
        return notes;
    }
    let _one_at_a_time = lock(&READING);
    if let Some((at, notes)) = lock(&SNAPSHOT).get(&snapshot_key).cloned()
        && at == epoch
    {
        return notes;
    }
    let notes = Arc::new(read(store, need_body).unwrap_or_default());
    lock(&SNAPSHOT).insert(snapshot_key, (epoch, notes.clone()));
    notes
}

fn read(store: &Store, need_body: bool) -> Option<Vec<ViewNote>> {
    store
        .query_view_notes_with_body(need_body)
        .ok()
        .map(|rows| rows.into_iter().map(ViewNote::from).collect())
}

/// Whether evaluating `spec` needs the note bodies: only free text and
/// `body:` clauses match against them.
fn needs_body(spec: &ViewSpec) -> bool {
    spec.from
        .clauses
        .iter()
        .any(|clause| clause.field.is_none() || clause.field == Some(kanso_core::Field::Body))
}

/// The rows for `spec`, once they are ready. The last rows stay on screen
/// while the notes are read again in the background.
fn rows_for(
    spec: &ViewSpec,
    source: &str,
    rich: &Rich,
    self_id: Option<String>,
    cx: &mut App,
) -> Option<Arc<Vec<ViewRow>>> {
    let store = store_key(&rich.store);
    settle_if_due(store);
    let key = (store, hash(source, self_id.as_deref()));
    let epoch = settled(store);
    let pending = pending_for(store);
    let mut stale = None;
    if let Some(ready) = lock(&RESULTS).get(&key) {
        // A pending change makes rows stale even while `settled` still
        // names them: the change has not been read yet.
        if ready.epoch == epoch && pending.is_none() {
            return Some(ready.rows.clone());
        }
        stale = Some(ready.rows.clone());
    }
    if let Some(delay) = pending {
        // Nothing to keep on screen yet: read now rather than show a blank.
        // With rows already drawn, wait for the burst of saves to settle,
        // then redraw and read.
        if stale.is_none() {
            start(spec, rich, self_id, key, epoch, cx);
        }
        wake_after(rich, delay, store, cx);
        return stale;
    }
    // Settled and out of date: read the notes again. The last rows stay on
    // screen meanwhile, and `WORKING` keeps one evaluation per change.
    start(spec, rich, self_id, key, epoch, cx);
    stale
}

/// Reads the notes and evaluates `spec` in the background, at `epoch`.
fn start(
    spec: &ViewSpec,
    rich: &Rich,
    self_id: Option<String>,
    key: CacheKey,
    epoch: u64,
    cx: &mut App,
) {
    let work = (key, epoch);
    if !lock(&WORKING).insert(work) {
        return;
    }
    let store = rich.store.clone();
    let wake = rich.wake.clone();
    let spec = spec.clone();
    let need_body = needs_body(&spec);
    cx.background_executor()
        .spawn(async move {
            let evaluated = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let notes = notes(epoch, &store, need_body);
                Arc::new(query_view::evaluate(&spec, &notes, self_id.as_deref()))
            }));
            if let Ok(rows) = evaluated {
                let mut results = lock(&RESULTS);
                if results.len() >= 256 {
                    results.clear();
                }
                // A slower, older evaluation never replaces newer rows.
                let newer = results.get(&key).is_some_and(|ready| ready.epoch > epoch);
                if !newer {
                    results.insert(key, Ready { epoch, rows });
                }
            }
            // Remove the key even when evaluation panicked, so it can try
            // again rather than waiting forever.
            lock(&WORKING).remove(&work);
            wake.try_send(()).ok();
        })
        .detach();
}

fn hash(source: &str, self_id: Option<&str>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    self_id.hash(&mut hasher);
    hasher.finish()
}

fn render(
    node: &MarkdownNode,
    rich: &Arc<Rich>,
    links: &LinkSender,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(block) = node.data::<QueryBlock>() else {
        return div().into_any_element();
    };
    let at = node.source_range().map_or(0, |r| r.start) as u64;
    let spec = match query_view::parse(&block.source) {
        Ok(spec) => spec,
        Err(err) => return bad_syntax(block, &err, at, window, cx),
    };
    let self_id = rich.note.lock().ok().and_then(|note| note.clone());
    let Some(rows) = rows_for(&spec, &block.source, rich, self_id, cx) else {
        return waiting(at, cx);
    };
    table(&spec, &rows, at, links, cx)
}

/// A query that can't be read: its source, with a badge saying why.
fn bad_syntax(
    block: &QueryBlock,
    err: &query_view::ParseError,
    at: u64,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let detail = err.message.clone();
    let badge = div()
        .id(ElementId::NamedInteger("kanso-query-badge".into(), at))
        .absolute()
        .top(px(7.))
        .left(px(10.))
        .px(px(7.))
        .py(px(1.))
        .rounded(px(5.))
        .bg(colors.fill)
        .border_1()
        .border_color(colors.divider)
        .text_size(px(11.5))
        .text_color(colors.text_secondary)
        .child("Query")
        .tooltip(move |window, cx| {
            gpui_kit::component::tooltip::Tooltip::new(detail.clone()).build(window, cx)
        });
    div()
        .relative()
        .w_full()
        .child(crate::preview::code_view(
            ElementId::NamedInteger("kanso-query-code".into(), at),
            block.markdown.clone(),
            window,
            cx,
        ))
        .child(badge)
        .into_any_element()
}

/// Room held while the notes are read.
fn waiting(at: u64, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    div()
        .id(ElementId::NamedInteger("kanso-query-waiting".into(), at))
        .w_full()
        .h(px(48.))
        .rounded(px(8.))
        .bg(Hsla::from(colors.fill).opacity(0.5))
        .into_any_element()
}

/// The table: a header, then a row for each note, its title first.
fn table(spec: &ViewSpec, rows: &[ViewRow], at: u64, links: &LinkSender, cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let labels = spec.labels();
    let mut table = v_flex()
        .id(ElementId::NamedInteger("kanso-query".into(), at))
        .debug_selector(|| "kanso-query".into())
        .w_full()
        .min_w_0()
        .pb(px(6.))
        .child(header(&labels, colors));
    if rows.is_empty() {
        table = table.child(
            div()
                .px(px(8.))
                .py(px(8.))
                .text_size(px(12.5))
                .text_color(colors.text_muted)
                .child("No notes match."),
        );
        return table.into_any_element();
    }
    for (ix, row) in rows.iter().enumerate() {
        let links = links.clone();
        let id = row.id.clone();
        table = table.child(
            h_flex()
                .id(ElementId::NamedInteger("kanso-query-row".into(), ix as u64))
                .debug_selector(move || format!("kanso-query-row-{ix}"))
                .w_full()
                .min_w_0()
                .gap(px(12.))
                .px(px(8.))
                .py(px(5.))
                .rounded(px(5.))
                .hover(|el| el.bg(colors.row_hover))
                .child(
                    div()
                        .id(ElementId::NamedInteger(
                            "kanso-query-title".into(),
                            ix as u64,
                        ))
                        .debug_selector(move || format!("kanso-query-title-{ix}"))
                        .flex_none()
                        .w(relative(0.34))
                        .min_w_0()
                        .truncate()
                        .text_size(px(13.))
                        .text_color(colors.link)
                        .cursor_pointer()
                        .hover(|el| el.text_color(colors.text))
                        .child(row.title.clone())
                        .on_click(move |_, _, _| {
                            links.try_send(id.clone()).ok();
                        }),
                )
                .children(row.cells.iter().map(|cell| {
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(cell.as_ref().map(cell_text).unwrap_or_default())
                })),
        );
    }
    table.into_any_element()
}

fn header(labels: &[String], colors: &Colors) -> AnyElement {
    h_flex()
        .w_full()
        .min_w_0()
        .gap(px(12.))
        .px(px(8.))
        .pb(px(4.))
        .border_b_1()
        .border_color(colors.divider)
        .children(labels.iter().enumerate().map(|(ix, label)| {
            div()
                .when(ix == 0, |el| el.flex_none().w(relative(0.34)))
                .when(ix > 0, |el| el.flex_1())
                .min_w_0()
                .truncate()
                .text_size(px(11.5))
                .text_color(colors.text_muted)
                .child(label.clone())
        }))
        .into_any_element()
}

/// A cell as the app writes it (see `kanso_core::query_view::cell_text`).
pub use kanso_core::query_view::cell_text;

#[cfg(test)]
mod tests {
    use super::cell_text;
    use kanso_core::properties::Value;

    #[test]
    fn only_free_text_queries_need_the_body() {
        use kanso_core::query_view;
        for (source, expected) in [
            ("from: status:active", false),
            ("from: has:due", false),
            ("columns: client", false),
            ("from: book:Clients", false),
            ("from: Acme", true),
            ("from: body:Acme", true),
            ("from: -Acme", true),
            ("from: -body:Acme", true),
        ] {
            let spec = query_view::parse(source).unwrap();
            assert_eq!(super::needs_body(&spec), expected, "{source}");
        }
    }

    #[test]
    fn cells_read_as_the_app_writes_them() {
        assert_eq!(cell_text(&Value::Text("Acme".into())), "Acme");
        assert_eq!(cell_text(&Value::Number(4.)), "4");
        assert_eq!(cell_text(&Value::Bool(true)), "✓");
        assert_eq!(cell_text(&Value::Bool(false)), "");
        assert_eq!(
            cell_text(&Value::Date("2026-11-01".into())),
            "Sunday, 1 November 2026"
        );
        assert_eq!(
            cell_text(&Value::List(vec![
                Value::Text("Ana".into()),
                Value::Text("Ben".into()),
            ])),
            "Ana, Ben"
        );
    }
}
