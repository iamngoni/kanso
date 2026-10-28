// Kanso — ai_status_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'ai_status_bloc.dart';

@freezed
abstract class AiStatusState with _$AiStatusState {
  const factory AiStatusState({
    /// Whether the settings have been read.
    @Default(false) bool loaded,
    @Default(AiSettings()) AiSettings settings,
  }) = _AiStatusState;

  const AiStatusState._();

  /// Whether the AI features are on.
  bool get enabled => settings.enabled;
}
