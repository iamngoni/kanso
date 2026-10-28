// Kanso — theme_picker_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'theme_picker_bloc.dart';

@freezed
abstract class ThemePickerState with _$ThemePickerState {
  const factory ThemePickerState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<ThemePreview>[]) List<ThemePreview> themes,

    /// The theme files that were left out, and why.
    @Default(<ThemeFileNotice>[]) List<ThemeFileNotice> notices,

    /// Whether a file is being imported.
    @Default(false) bool importing,

    /// How the last import ended; null before one, and while another runs.
    ThemeImportOutcome? outcome,
  }) = _ThemePickerState;
}
