// Kanso — daily_note_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'daily_note_bloc.dart';

@freezed
abstract class DailyNoteState with _$DailyNoteState {
  const factory DailyNoteState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// Today's note, once [status] is ready.
    String? noteId,

    /// Why it could not be opened, once [status] is failed.
    ApplicationError? error,
  }) = _DailyNoteState;
}
