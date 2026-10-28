// Kanso — assist_snapshot.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'assist_snapshot.freezed.dart';

/// The note's text as it was when the assistant was opened, and what was
/// selected in it, so a suggestion is applied only if that text still is
/// what it was written for.
@freezed
abstract class AssistSnapshot with _$AssistSnapshot {
  const factory AssistSnapshot({
    /// The note the editor was showing.
    required String noteId,

    /// Changes on every edit, reload, or selection update.
    required int revision,

    /// The current note-window origin, in UTF-16 units.
    required int windowOrigin,

    /// The editor's whole text then.
    required String text,

    /// The selection then; a caret when nothing was selected.
    required TextRange selection,
  }) = _AssistSnapshot;

  const AssistSnapshot._();

  /// The selected text; empty for a caret.
  String get selected => selection.textInside(text);

  /// The text before the selection.
  String get before => selection.textBefore(text);
}
