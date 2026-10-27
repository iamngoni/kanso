// Kanso — lock_target.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'lock_target.freezed.dart';

/// What is being locked or opened: one note, or one notebook.
@freezed
sealed class LockTarget with _$LockTarget {
  const factory LockTarget.note(String id) = NoteLockTarget;

  /// The notebook [id], with its [name] for the words that mention it.
  const factory LockTarget.notebook(String id, String name) =
      NotebookLockTarget;
}
