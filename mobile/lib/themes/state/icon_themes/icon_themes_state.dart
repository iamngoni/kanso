// Kanso — icon_themes_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'icon_themes_bloc.dart';

@freezed
abstract class IconThemesState with _$IconThemesState {
  const factory IconThemesState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The imported and gallery icon themes.
    @Default(<InstalledIconTheme>[]) List<InstalledIconTheme> themes,

    /// Whether a file is being imported.
    @Default(false) bool importing,

    /// How the last import ended; null before one, and while another runs.
    ThemeImportOutcome? outcome,
  }) = _IconThemesState;
}
