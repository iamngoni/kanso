// Kanso — daily_template_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'daily_template_bloc.dart';

@freezed
abstract class DailyTemplateState with _$DailyTemplateState {
  const factory DailyTemplateState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<TemplateRow>[]) List<TemplateRow> templates,

    /// The id of the template chosen; null for none.
    String? current,

    /// Why reading or setting failed.
    ApplicationError? error,
  }) = _DailyTemplateState;

  const DailyTemplateState._();

  /// The chosen template's row, if there is one.
  TemplateRow? get chosen {
    for (final row in templates) {
      if (row.id == current) {
        return row;
      }
    }
    return null;
  }
}
