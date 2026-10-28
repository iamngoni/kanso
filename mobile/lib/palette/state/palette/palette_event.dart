// Kanso — palette_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'palette_bloc.dart';

@freezed
sealed class PaletteEvent with _$PaletteEvent {
  /// The palette opened, with what the app can do now in [context].
  const factory PaletteEvent.started(PaletteContext context) =
      PaletteStartedEvent;

  /// What is typed changed to [query].
  const factory PaletteEvent.queryChanged(String query) =
      PaletteQueryChangedEvent;

  /// Typing rested: look again for [query].
  const factory PaletteEvent.searchDue(String query) = PaletteSearchDueEvent;
}
