// Kanso — capture_task_state.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'capture_task_cubit.dart';

@freezed
abstract class CaptureTaskState with _$CaptureTaskState {
  const factory CaptureTaskState({
    /// Whether the saved choices have been read.
    @Default(false) bool loaded,

    /// The mode the capture sheet is in; the last one chosen.
    @Default(CaptureMode.note) CaptureMode mode,

    /// Where a captured task is written.
    @Default(CaptureTarget.dailyNote) CaptureTarget target,

    /// How the date and repeat at the end of the typed text read, or null.
    String? hint,

    /// Where the task being saved, or the last one, has got to.
    @Default(CaptureTaskStatus.idle) CaptureTaskStatus status,
  }) = _CaptureTaskState;
}
