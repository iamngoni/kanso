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
  }) = _AskState;

  const AskState._();

  /// Whether a question is being answered.
  bool get busy => phase != AskPhase.idle;
}
