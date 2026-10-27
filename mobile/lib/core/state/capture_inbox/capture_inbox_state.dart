// Kanso — capture_inbox_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'capture_inbox_bloc.dart';

@freezed
abstract class CaptureInboxState with _$CaptureInboxState {
  const factory CaptureInboxState({
    /// Whether a save is running.
    @Default(false) bool saving,

    /// The IDs of the notes the last save made.
    @Default(<String>[]) List<String> saved,

    /// How many notes saves have made since the app started.
    @Default(0) int savedCount,

    /// How many items the last save kept to try again.
    @Default(0) int kept,

    /// What was left out or refused since the person last dismissed them, a
    /// sentence each, for the screen to show as a short notice.
    @Default(<String>[]) List<String> problems,

    /// Why the last save failed; null once one works.
    ApplicationError? error,
  }) = _CaptureInboxState;
}
