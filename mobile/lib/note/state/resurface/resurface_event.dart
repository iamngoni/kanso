// Kanso — resurface_event.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'resurface_bloc.dart';

@freezed
sealed class ResurfaceEvent with _$ResurfaceEvent {
  /// A note was opened.
  const factory ResurfaceEvent.started(String noteId) = ResurfaceStartedEvent;

  /// The text or the caret changed. [caret] is a UTF-16 offset in [body].
  const factory ResurfaceEvent.edited({
    required String title,
    required String body,
    required int caret,
  }) = ResurfaceEditedEvent;

  /// "Not related": this note is not offered for the open one again.
  const factory ResurfaceEvent.dismissed(String otherId) =
      ResurfaceDismissedEvent;
}
