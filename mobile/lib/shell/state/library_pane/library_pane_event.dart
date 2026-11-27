// Kanso — library_pane_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'library_pane_bloc.dart';

@freezed
sealed class LibraryPaneEvent with _$LibraryPaneEvent {
  /// The person asked to show the pane, or to hide it.
  const factory LibraryPaneEvent.toggled() = LibraryPaneToggledEvent;

  /// The person asked to put the library and the list away beside an open
  /// note, or to bring them back.
  const factory LibraryPaneEvent.panesToggled() = LibraryPanesToggledEvent;
}
