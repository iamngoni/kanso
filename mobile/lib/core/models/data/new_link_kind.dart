// Kanso — new_link_kind.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What `kanso://open/new` (the `/new` address) starts, from a launcher
/// shortcut, the widget, a Quick Settings tile or another app.
enum NewLinkKind {
  /// A new note, open to write its title (`kanso://open/new`).
  note,

  /// A new note that starts in its text, to dictate into (`?voice=1`).
  voice,

  /// The capture sheet in Task mode, for one task (`?task=1`).
  task,
}
