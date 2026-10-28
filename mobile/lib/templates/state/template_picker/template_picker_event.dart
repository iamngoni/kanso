// Kanso — template_picker_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'template_picker_bloc.dart';

@freezed
sealed class TemplatePickerEvent with _$TemplatePickerEvent {
  /// The picker was opened (or tried again): read the templates.
  const factory TemplatePickerEvent.started() = TemplatePickerStartedEvent;
}
