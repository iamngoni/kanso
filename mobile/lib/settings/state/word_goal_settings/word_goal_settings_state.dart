// Kanso — word_goal_settings_state.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'word_goal_settings_bloc.dart';

@freezed
abstract class WordGoalSettingsState with _$WordGoalSettingsState {
  const factory WordGoalSettingsState({
    @Default(LoadStatus.loading) LoadStatus status,
    int? words,
    ApplicationError? error,
  }) = _WordGoalSettingsState;
}
