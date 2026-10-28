// Kanso — template_picker_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'template_picker_bloc.dart';

@freezed
abstract class TemplatePickerState with _$TemplatePickerState {
  const factory TemplatePickerState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<TemplateRow>[]) List<TemplateRow> templates,
    ApplicationError? error,
  }) = _TemplatePickerState;
}
