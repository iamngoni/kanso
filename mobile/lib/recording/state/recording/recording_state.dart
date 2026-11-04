// Kanso — recording_state.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recording_bloc.dart';

/// Where the recorder stands.
enum RecordingPhase { idle, starting, recording, saving }

/// Why a recording did not start or was not kept.
enum RecordingProblem {
  /// Another recording is running.
  busy,

  /// The microphone is denied, missing or taken.
  cannotStart,

  /// The note is locked: unlock it first.
  locked,

  /// The note is gone.
  noteGone,

  /// The audio is kept in its folder; Recover offers it again.
  saveFailed,
}

@freezed
abstract class RecordingState with _$RecordingState {
  const factory RecordingState({
    @Default(RecordingPhase.idle) RecordingPhase phase,

    /// Audio time recorded, pause excluded.
    @Default(0) int seconds,

    /// The level meter, 0 to 1.
    @Default(0.0) double level,
    @Default(false) bool paused,

    /// The 5-hour warning has been raised.
    @Default(false) bool warned,

    /// The note being recorded into; null for a new note.
    String? noteId,

    /// How the last recording ended by itself (cap, microphone, disk).
    RecordingEnd? ended,

    /// The last recording's note, until the screen has shown it.
    RecordingSaved? saved,
    RecordingProblem? problem,

    /// Recordings a crash left.
    @Default(<RecordingFound>[]) List<RecordingFound> found,
  }) = _RecordingState;

  const RecordingState._();

  bool get active =>
      phase == RecordingPhase.recording || phase == RecordingPhase.saving;
}
