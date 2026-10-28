// Kanso — daily_template_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'daily_template_bloc.dart';

@freezed
sealed class DailyTemplateEvent with _$DailyTemplateEvent {
  /// Settings opened: read the templates and the one chosen.
  const factory DailyTemplateEvent.started() = DailyTemplateStartedEvent;

  /// The person chose [templateId], or none (null).
  const factory DailyTemplateEvent.chosen(String? templateId) =
      DailyTemplateChosenEvent;

  const factory DailyTemplateEvent.errorDismissed() =
      DailyTemplateErrorDismissedEvent;
}
