// Kanso — daily_note_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'daily_note_bloc.dart';

@freezed
sealed class DailyNoteEvent with _$DailyNoteEvent {
  /// Today's note is wanted, on the phone's local date and time [now]. Also
  /// what trying again sends, with the time then.
  const factory DailyNoteEvent.requested(DateTime now) =
      DailyNoteRequestedEvent;
}
