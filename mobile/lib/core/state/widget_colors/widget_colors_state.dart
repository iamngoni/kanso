// Kanso — widget_colors_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'widget_colors_bloc.dart';

@freezed
abstract class WidgetColorsState with _$WidgetColorsState {
  const factory WidgetColorsState({
    /// How many times the colours were written since the app started.
    @Default(0) int publishes,

    /// Why the last attempt failed; null once one works.
    ApplicationError? error,
  }) = _WidgetColorsState;
}
