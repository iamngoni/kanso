// Kanso — notebook_lock_rule.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';

/// What the notebook's menu can do about its lock.
enum NotebookLockRule {
  /// It isn't locked: it can be locked.
  lock,

  /// It has a lock of its own: it can be removed.
  remove,

  /// It is locked through a notebook it is inside: nothing here.
  none,
}

/// Whether a notebook above [notebook] in [all] has a lock of its own.
bool lockedThroughParent(NotebookRow notebook, List<NotebookRow> all) {
  final byId = <String, NotebookRow>{for (final book in all) book.id: book};
  var parentId = notebook.parentId;
  // A guard against a cycle in the data.
  for (var step = 0; parentId != null && step < all.length; step++) {
    final parent = byId[parentId];
    if (parent == null) {
      return false;
    }
    if (parent.locked) {
      return true;
    }
    parentId = parent.parentId;
  }
  return false;
}

/// The desktop's rule: Remove Notebook Lock when the notebook has a lock of
/// its own, Lock Notebook when it isn't locked, nothing when it is locked
/// through a parent notebook.
NotebookLockRule notebookLockRule(NotebookRow notebook, List<NotebookRow> all) {
  if (notebook.locked) {
    return NotebookLockRule.remove;
  }
  return lockedThroughParent(notebook, all)
      ? NotebookLockRule.none
      : NotebookLockRule.lock;
}
