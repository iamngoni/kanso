// Kanso — journal_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'journal_bloc.dart';

@freezed
abstract class JournalState with _$JournalState {
  const factory JournalState({
    /// Whether a note is being found or made.
    @Default(false) bool working,

    /// The note to open; null until one is made, and again while the next
    /// is being made.
    MadeNote? opened,

    /// Why the last attempt failed, until it is dismissed.
    ApplicationError? error,
  }) = _JournalState;
}
