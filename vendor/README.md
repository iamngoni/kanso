# Vendored crates

Kanso builds against the crates.io release of GPUI Kit, pinned in the
workspace `Cargo.toml` (`gpui-kit = "=0.7.3"`), with one crate replaced by
the copy here through `[patch.crates-io]`:

| Crate | Version | Why |
|---|---|---|
| [`gpui-component`](gpui-component) | 0.7.3, as published | One small hook, so an icon theme draws the component library's own icons too. |
| [`gpui-base`](gpui-base) | 0.7.3, as published | One small hook, so a click on a task box in a Markdown view says which task it was; and a render-only presentation for the code editor (hidden and replaced ranges, taller, spaced, inset and ornamented lines, block widgets) for Live mode. |

## What changed

Three files differ from the published crate (the full diff is in
[UPSTREAM.md](UPSTREAM.md), written to be offered upstream), and a fourth,
`src/highlighter/highlighter.rs`, gains `set_injection_parse_timeout`: it
replaces the 20 ms deadline of an injection parse, so a test comparing this
highlighter with another's isn't cut short on a loaded machine (unset, the
deadline is unchanged; not part of the upstream proposal):

- `src/icon.rs` adds `IconResolver`, a global holding a function from an
  icon's asset path to the path to draw instead, `set_icon_resolver` to set
  it, and `resolve_icon_path` to ask it. An `Icon` with a path asks it as it
  renders (both its `RenderOnce` and `Render`), so every icon drawn through
  `Icon` (buttons, menus, selects, dialogs, notifications, date pickers…)
  follows it. With no resolver set nothing changes.
- `src/checkbox.rs` and `src/questionnaire/components.rs` draw the check
  with `svg()` directly rather than through `Icon`; they ask the resolver
  too.

Nothing else is touched: the crate's `Cargo.toml`, features and everything
else are the published files (its `Cargo.lock` and `.cargo-ok` are left
out, as Cargo ignores them in a path dependency).

## gpui-base: which task box was clicked

Three files differ from the published `gpui-base` (the diff is in
[UPSTREAM-gpui-base.md](UPSTREAM-gpui-base.md)):

- `src/text/text_view.rs` adds `TaskClickHandlerFn`, called with the byte
  offset in the source of the task item whose box was clicked, and a
  `task_click_handler` on `TextView` and its builder, in the same shape as
  the existing `link_click_handler`.
- `src/text/state.rs` carries the handler through `TextViewStyle`.
- `src/text/node.rs` makes the box of a task item clickable when a handler
  is set, and reports that item's byte offset. With no handler the box draws
  exactly as before and clicks do nothing.

Nothing else is touched: `Cargo.toml`, features and the rest are the
published files.

## gpui-base: a render-only presentation for the code editor

Live mode draws a note's Markdown in place: marks such as `**` and `#` are
hidden on the lines the caret is not on, headings are drawn larger, list
markers become bullets, quotes get a bar, and blocks such as tables can be
drawn by the Preview's renderer. The
editor's text stays the Markdown source, so this is a presentation of it,
set with `EditorState::set_presentation` (off by default; with none set the
editor lays out exactly as before). The diff and the proposal are in
[UPSTREAM-gpui-base-presentation.md](UPSTREAM-gpui-base-presentation.md).
These files differ from the published `gpui-base` for it:

- `src/input/editor/presentation.rs` (new) defines `EditorPresentation`
  (concealed byte ranges, per-line scales, replacements, per-line space,
  insets and ornaments, block widgets) and `EditorPresentationEvent` (a
  clickable replacement or a block widget was clicked), validates it against
  the text, and adds `set_presentation` and `editor_presentation` to
  `EditorState`. `src/input/mod.rs` declares it and re-exports its types.
- `src/input/editor/display_map/text_wrapper.rs` keeps the presentation,
  carries it across edits, wraps a concealed range as a zero-width element, a
  replacement as an element as wide as its text, a scaled line at
  `wrap_width / scale`, an inset line that much narrower and a block widget's
  first line not at all, and gives `LineLayout` a scale and a frame (space
  above and below, inset, block height).
- `src/input/editor/display_map/wrap_map.rs` and `display_map.rs` pass it
  through, and the display map keeps the rows that are not one line height
  (scaled lines, spaced lines, block widgets, with measured widget heights) to
  map display rows to and from y (`display_row_top`, `display_row_at_y`,
  `display_row_height`, `wrap_rows_height`). `fold_map.rs` hides a block
  widget's lines after the first the way a fold hides its inside.
- `src/input/editor/display_map/inline_line.rs`: a click on a zero-width
  fragment lands after it, a replacement fragment is atomic for the caret,
  and `is_concealed` tells whitespace indicators to skip hidden text. A
  replacement fragment can be drawn offset in its room (right-aligned in a
  column, or hanging left of it into the inset), moved to keep the row's
  baseline at another size, on a box (with an icon and a dashed underline),
  or as an image.
- Live mode draws as the Preview does, so the presentation also has a row
  height factor per line (`line_heights`, a heading's tighter line height),
  a base row height that is not rounded (`line_height`), scales below 1.0,
  replacements in their own family and size, in columns, on chips or as
  images, and a check box ornament (`Ornament::Check`). It also has
  `leaves`: runs of lines laid out as one box whose height is rounded up to
  whole device pixels, as the layout rounds a measured text box. Chips that
  stand for an inline element sit on whole device pixels, with their text
  placed in the box as the element places it.
  `src/input/base/element.rs` lays these out and paints them, and clips
  decorations to the visible lines when there is no highlighter (a
  decoration over a line hidden by a block widget shifted every later run).
- `src/input/base/element.rs` shapes a presented line at its scale with its
  concealed ranges as zero-width fragments and its replacements as their
  text, uses the per-line heights for the visible range, scroll size, carets,
  selections and range decorations, line numbers, the active line and fold
  icons, paints ornaments under each line, and lays out and paints block
  widgets' elements in their rows.
- `src/input/base/state.rs`, `movement.rs`, `touch.rs` and
  `src/input/editor/indent.rs` use the per-line heights for hit testing,
  scrolling to the caret, IME and range bounds, up/down moves, touch handles
  and indent guides; a press on a clickable replacement is reported instead
  of placing the caret.

## Why

GPUI keeps an SVG it has rasterized, per window, by its asset path and
size, and an app can't clear that cache. Kanso's icon themes
(`crates/kanso-app/src/icon_themes.rs`) switch every icon at once by giving
each themed icon a path of its own (`icons/@<theme>-<fingerprint>/<name>.svg`),
which Kanso's asset source loads. Kanso's own icons go through `Icon`, and
so do the component library's, but only the library can rewrite the paths
of the icons it builds inside its controls; the hook lets it.

Not covered: icons the library hands to the system's native menus
(`native_menu`) keep their paths; Kanso's menus have no icons.

## Upgrading GPUI Kit

1. Bump the pins in the workspace `Cargo.toml` (and the `gpui-component` dev-dependency in
   `crates/kanso-app/Cargo.toml`).
2. If the new `gpui-component` has an icon hook of its own (see
   UPSTREAM.md), remove the `[patch.crates-io]` entry and this folder, and
   call the library's hook from `icon_themes::init`.
3. Otherwise copy the new release in: `cp -R
   ~/.cargo/registry/src/*/gpui-component-<version> vendor/gpui-component`
   (after `cargo fetch`), delete its `Cargo.lock` and `.cargo-ok`, and apply
   the diff in UPSTREAM.md (`patch -p1 -d vendor/gpui-component <
   the-diff`); fix up by hand where the files have moved on, and look for new
   `svg().path(IconName::…)` calls (`grep -rn "svg()" src`) to route through
   `resolve_icon_path` too.
4. `cargo test -p kanso-app switching_icon_theme` checks a checkbox's check
   follows the icon theme in a window.
