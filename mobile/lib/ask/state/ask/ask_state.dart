// Kanso — ask_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ask_bloc.dart';

@freezed
abstract class AskState with _$AskState {
  const factory AskState({
    /// Whether the settings have been read.
    @Default(LoadStatus.loading) LoadStatus status,

    /// Whether AI is on; with it off there is nothing to ask.
    @Default(false) bool aiEnabled,

    /// The line about where a question goes, shown while the conversation is
    /// empty.
    @Default('') String privacy,

    /// The conversation, oldest question first.
    @Default(<AskTurn>[]) List<AskTurn> turns,
    @Default(AskPhase.idle) AskPhase phase,

    /// What the next question searches.
    @Default(AskScope.everything) AskScope scope,

    /// What a question can be limited to, once the scope sheet asked.
    @Default(<AskScope>[]) List<AskScope> scopes,

    /// Why the answer could not be saved, in plain words.
    String? saveFailure,
  }) = _AskState;

  const AskState._();

  /// Whether a question is being answered.
  bool get busy => phase != AskPhase.idle;
}
