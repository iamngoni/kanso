// Kanso — assist_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'assist_bloc.dart';

@freezed
abstract class AssistState with _$AssistState {
  const factory AssistState({
    @Default(AssistStatus.idle) AssistStatus status,

    /// What was asked for; null before the person has chosen.
    AssistAction? action,

    /// Whether the suggestion is for the whole note (its summary) and not
    /// for text selected in it.
    @Default(false) bool wholeNote,

    /// The suggestion so far, or whole once [status] is ready.
    @Default('') String suggestion,

    /// Whether the suggestion was not applied because the note's text changed
    /// since it was asked for.
    @Default(false) bool changed,

    /// Why the request failed, in plain words, while [status] is failed.
    String? failure,
  }) = _AssistState;
}
