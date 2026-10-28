// Kanso — widget_colors_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'widget_colors_bloc.dart';

@freezed
sealed class WidgetColorsEvent with _$WidgetColorsEvent {
  /// The app starts, or its light or dark theme changed: these are the
  /// colours the widgets are to draw with.
  const factory WidgetColorsEvent.changed({
    required KansoColors light,
    required KansoColors dark,
  }) = WidgetColorsChangedEvent;
}
