// Kanso — undo_notice.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'undo_notice.freezed.dart';

/// A change the person can take back, for the few seconds the notice that
/// offers it shows.
@freezed
abstract class UndoNotice with _$UndoNotice {
  const factory UndoNotice({
    /// The ticket the library keeps the change under (`Library::undo`).
    required int ticket,

    /// What the notice says, as the library words it.
    required String message,

    /// Which notice this is: a later one has a larger number, so a timeout
    /// belongs to the notice it was started for.
    required int serial,
  }) = _UndoNotice;
}
