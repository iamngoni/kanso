// Kanso — word_goal_settings_event.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'word_goal_settings_bloc.dart';

@Freezed(copyWith: false)
sealed class WordGoalSettingsEvent with _$WordGoalSettingsEvent {
  const factory WordGoalSettingsEvent.started() = WordGoalSettingsStartedEvent;

  const factory WordGoalSettingsEvent.defaultChanged(int? words) =
      WordGoalDefaultChangedEvent;
}
