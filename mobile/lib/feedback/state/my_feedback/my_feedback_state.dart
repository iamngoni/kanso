// Kanso — my_feedback_state.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'my_feedback_bloc.dart';

@freezed
abstract class MyFeedbackState with _$MyFeedbackState {
  const factory MyFeedbackState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The reports, newest first.
    @Default(<FeedbackEntry>[]) List<FeedbackEntry> entries,

    /// The server is being asked.
    @Default(false) bool refreshing,
  }) = _MyFeedbackState;

  const MyFeedbackState._();

  /// The newest report, for the Settings row's value.
  FeedbackEntry? get newest => entries.isEmpty ? null : entries.first;

  /// The release of the newest report that was fixed and not yet looked at.
  String? get freshRelease {
    for (final entry in entries) {
      if (entry.fresh && entry.release != null) {
        return entry.release;
      }
    }
    return null;
  }
}
