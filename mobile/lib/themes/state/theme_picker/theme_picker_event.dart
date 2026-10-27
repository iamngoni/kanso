// Kanso — theme_picker_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'theme_picker_bloc.dart';

@freezed
sealed class ThemePickerEvent with _$ThemePickerEvent {
  /// The picker was opened, or the person asked to try again: read the
  /// themes.
  const factory ThemePickerEvent.started() = ThemePickerStartedEvent;

  /// The person asked to import a theme: pick a file, then import it.
  const factory ThemePickerEvent.importRequested() =
      ThemePickerImportRequestedEvent;
}
