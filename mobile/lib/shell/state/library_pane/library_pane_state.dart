// Kanso — library_pane_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'library_pane_bloc.dart';

@freezed
abstract class LibraryPaneState with _$LibraryPaneState {
  const factory LibraryPaneState({
    /// Whether the pane is shown beside the list.
    @Default(true) bool visible,

    /// Whether the library and the list are both put away while a note is
    /// open, so the note has the whole width of a tablet to write in.
    @Default(false) bool panesHidden,
  }) = _LibraryPaneState;
}
