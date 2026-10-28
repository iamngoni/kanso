// Kanso — focus_mode_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'focus_mode_bloc.dart';

@freezed
sealed class FocusModeEvent with _$FocusModeEvent {
  /// The app started: read the choice kept in the preferences.
  const factory FocusModeEvent.started() = FocusModeStartedEvent;

  /// The person asked to enter focus mode, or to leave it.
  const factory FocusModeEvent.toggled() = FocusModeToggledEvent;

  /// The person asked to leave focus mode.
  const factory FocusModeEvent.exited() = FocusModeExitedEvent;
}
