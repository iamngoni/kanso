// Kanso — icon_themes_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'icon_themes_bloc.dart';

@freezed
sealed class IconThemesEvent with _$IconThemesEvent {
  /// The page was opened, or the person asked to try again: read the icon
  /// themes.
  const factory IconThemesEvent.started() = IconThemesStartedEvent;

  /// The person asked to import an icon theme: pick a file, then import it.
  const factory IconThemesEvent.importRequested() =
      IconThemesImportRequestedEvent;
}
