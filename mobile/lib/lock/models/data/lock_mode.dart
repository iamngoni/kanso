// Kanso — lock_mode.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How locking something goes, as the desktop does it.
enum LockMode {
  /// No lock password exists yet: make one, typed twice.
  setPassword,

  /// The lock password exists but the notes are closed: ask for it.
  askPassword,

  /// The notes are open: only confirm.
  confirm,
}
