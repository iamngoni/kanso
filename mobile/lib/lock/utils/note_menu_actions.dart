// Kanso — note_menu_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../models/data/note_menu_action.dart';

/// The lock actions a note's menu offers, in order.
///
/// A note that was never locked can be locked. A locked note that is open
/// can have its lock removed. Lock notes now is there whenever they are
/// open. A locked note that is closed offers nothing: its card unlocks it.
List<NoteMenuAction> noteMenuActions({
  required LockState lock,
  required bool unlocked,
}) {
  return switch (lock) {
    LockState_Plain() => <NoteMenuAction>[
      NoteMenuAction.lockNote,
      if (unlocked) NoteMenuAction.lockNow,
    ],
    LockState_Open() => <NoteMenuAction>[
      NoteMenuAction.removeLock,
      NoteMenuAction.lockNow,
    ],
    LockState_Closed() => const <NoteMenuAction>[],
  };
}
