# Vendored crates

Kanso builds against the crates.io release of GPUI Kit, pinned in the
workspace `Cargo.toml` (`gpui-kit = "=0.7.3"`), with one crate replaced by
the copy here through `[patch.crates-io]`:

| Crate | Version | Why |
|---|---|---|
| [`gpui-component`](gpui-component) | 0.7.3, as published | One small hook, so an icon theme draws the component library's own icons too. |
| [`gpui-base`](gpui-base) | 0.7.3, as published | One small hook, so a click on a task box in a Markdown view says which task it was. |

## What changed

Three files differ from the published crate (the full diff is in
[UPSTREAM.md](UPSTREAM.md), written to be offered upstream):

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
