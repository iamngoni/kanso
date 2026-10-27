# Proposal: a task-box click hook for gpui-base

This is a change to offer to [GPUI Kit](https://github.com/longbridge/gpui-kit)
(`crates/base`), written against gpui-base 0.7.3. It hasn't been sent
anywhere.

## Summary

Add a handler that a `TextView` set to render Markdown can be given, called
with the byte offset in the source of the task-list item whose box was
clicked. The box becomes clickable only when a handler is set; with none,
the box draws and behaves exactly as it does today.

## Motivation

A Markdown view that shows a note's task list is the natural place to tick a
task, and the library already knows which item a click landed on: the box is
drawn per task item, with the item's own range at hand. Without a hook an app
can only overlay its own hit areas on top of the rendered text and guess,
which goes wrong with wrapped lines, scrolled content and the preview's own
link rewriting.

The offset, not the task's index, is what the app needs. The library counts
task items only among those it renders, while an app's own model of the note
may include tasks in front matter, in fenced code, or above a collapsed
section. Handing over the byte offset lets the app apply its own rule for
turning a box into a task, and refuse when the two disagree.

## The change

`src/text/text_view.rs`:

```rust
/// Called with the byte offset, in the view's source, of the task-list item
/// whose box was clicked.
pub(crate) type TaskClickHandlerFn = dyn Fn(usize, &mut Window, &mut App) + Send + Sync;
```

plus a `task_click_handler` field on `TextView` and its builder, mirroring
the existing `link_click_handler`, and carrying it through `TextViewStyle`
in `src/text/state.rs`.

`src/text/node.rs` makes the check mark of a task item a clickable element
when a handler is set, and calls it with the item's byte offset, using the
same `MouseButton::Left` press handling the note's links already use.

## Compatibility

Additive. `TextView::link_click_handler` and every other builder method keep
their signatures; a view built without the new method has no handler, and
the task boxes stay inert.

## Diff

The complete diff against the published crate is in this repository's
history for `vendor/gpui-base`; the three files it touches are listed in
[README.md](README.md).
