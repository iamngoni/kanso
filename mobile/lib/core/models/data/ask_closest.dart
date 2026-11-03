// Kanso — ask_closest.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ask_closest.freezed.dart';

/// A passage found for a question the notes do not answer.
@freezed
abstract class AskClosest with _$AskClosest {
  const factory AskClosest({
    required String noteId,
    @Default('') String title,
    @Default(0) int start,
    @Default(0) int end,
  }) = _AskClosest;
}
