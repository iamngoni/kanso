// Kanso — icon_themes_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'icon_themes_bloc.dart';

@freezed
abstract class IconThemesState with _$IconThemesState {
  const factory IconThemesState({
    @Default(LoadStatus.loading) LoadStatus status,
    @Default(<IconThemeRow>[]) List<IconThemeRow> themes,
  }) = _IconThemesState;
}
