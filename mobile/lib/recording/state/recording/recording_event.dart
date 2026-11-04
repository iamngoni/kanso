// Kanso — recording_event.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recording_bloc.dart';

@freezed
sealed class RecordingEvent with _$RecordingEvent {
  /// Start recording into a new note, or into [noteId].
  const factory RecordingEvent.started({String? noteId}) =
      RecordingStartedEvent;

  /// The clock: read the recorder.
  const factory RecordingEvent.ticked() = RecordingTickedEvent;

  const factory RecordingEvent.paused() = RecordingPausedEvent;

  const factory RecordingEvent.resumed() = RecordingResumedEvent;

  /// Stop and keep the recording.
  const factory RecordingEvent.stopped() = RecordingStoppedEvent;

  /// Stop and delete the recording (the screen has asked once).
  const factory RecordingEvent.discarded() = RecordingDiscardedEvent;

  /// The app started: look for a recording a crash left.
  const factory RecordingEvent.foundChecked() = RecordingFoundCheckedEvent;

  const factory RecordingEvent.recovered(String id) = RecordingRecoveredEvent;

  const factory RecordingEvent.foundDiscarded(String id) =
      RecordingFoundDiscardedEvent;

  /// The screen has shown the problem or the saved note.
  const factory RecordingEvent.noticeCleared() = RecordingNoticeClearedEvent;
}
