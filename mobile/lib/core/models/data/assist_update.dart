// Kanso — assist_update.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'assist_update.freezed.dart';

/// What the core tells the screen while the assistant writes a suggestion or
/// a note's summary.
@freezed
sealed class AssistUpdate with _$AssistUpdate {
  /// The suggestion written so far: all of it, not only the new part.
  const factory AssistUpdate.text({required String text}) = AssistText;

  /// The suggestion is complete.
  const factory AssistUpdate.done({required String suggestion}) = AssistDone;

  /// The note is kept out of AI (locked, in the trash or tagged
  /// `#ai-exclude`); nothing was sent.
  const factory AssistUpdate.keptOut() = AssistKeptOut;

  /// The request failed, in [message]'s plain words.
  const factory AssistUpdate.error({required String message}) = AssistFailed;
}
