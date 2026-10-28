// Kanso — tabs_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'tabs_bloc.dart';

@freezed
abstract class TabsState with _$TabsState {
  const factory TabsState({
    /// The open tabs, the one showing, and the ones that can be reopened.
    @Default(OpenTabs()) OpenTabs tabs,

    /// Whether the window has panes for tabs. Off, the tabs are dormant: a
    /// phone has none, and navigation is the app's one history.
    @Default(false) bool enabled,

    /// The note the tabs want shown, for the screen to open in the place
    /// where the person is; set when a tab was chosen, closed, reopened or
    /// stepped, never for a note the address already shows.
    String? pendingNoteId,
  }) = _TabsState;

  const TabsState._();

  /// Whether the bar is drawn: the desktop shows it from two tabs.
  bool get barVisible => enabled && tabs.length >= 2;
}
