// Kanso — preview_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'preview_bloc.dart';

@freezed
abstract class PreviewState with _$PreviewState {
  const factory PreviewState({
    /// The note drawn.
    @Default('') String noteId,

    /// The built-in theme it is drawn in.
    @Default('') String themeId,

    /// The page, once it is drawn; the last one stays while it is drawn
    /// again.
    String? html,

    /// The revision of the note the page was drawn from: a tick is made
    /// against it.
    @Default('') String rev,

    /// Why the page could not be drawn, when it couldn't.
    ApplicationError? error,

    /// What the screen is to do for the last tap, until it says it has.
    PreviewAction? action,

    /// Why the last tap did nothing, until it is dismissed.
    ApplicationError? notice,
  }) = _PreviewState;
}
