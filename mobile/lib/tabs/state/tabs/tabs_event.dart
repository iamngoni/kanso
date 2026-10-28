// Kanso — tabs_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'tabs_bloc.dart';

@freezed
sealed class TabsEvent with _$TabsEvent {
  /// The app started: restore the tabs kept in the preferences, and follow
  /// the library for notes that change.
  const factory TabsEvent.started() = TabsStartedEvent;

  /// The window now has room for tabs ([enabled]: two or three panes) or no
  /// longer has. [shownNoteId] is the note the address shows, if any.
  const factory TabsEvent.enabledChanged({
    required bool enabled,
    String? shownNoteId,
  }) = TabsEnabledChangedEvent;

  /// The note [noteId] is showing, some way other than the tabs: a tap in
  /// the list, a link, an address. It opens in the showing tab, unless it is
  /// open in another, which shows. Sent by the navigation history.
  const factory TabsEvent.noteOpened(String noteId) = TabsNoteOpenedEvent;

  /// Open [noteId] in a new tab after the showing one (or show its tab).
  const factory TabsEvent.openedInNewTab(String noteId) =
      TabsOpenedInNewTabEvent;

  /// Show the tab at [index].
  const factory TabsEvent.activated(int index) = TabsActivatedEvent;

  /// Close the tab at [index] (never the only one).
  const factory TabsEvent.closed(int index) = TabsClosedEvent;

  /// Close every tab but the one at [index].
  const factory TabsEvent.othersClosed(int index) = TabsOthersClosedEvent;

  /// Close every tab to the right of the one at [index].
  const factory TabsEvent.rightClosed(int index) = TabsRightClosedEvent;

  /// Move the tab at [from] to [to].
  const factory TabsEvent.reordered(int from, int to) = TabsReorderedEvent;

  /// Bring back the tab closed last, if its note is still there.
  const factory TabsEvent.reopenRequested() = TabsReopenRequestedEvent;

  /// Show the next tab, wrapping past the last.
  const factory TabsEvent.nextRequested() = TabsNextRequestedEvent;

  /// Show the tab before, wrapping from the first.
  const factory TabsEvent.previousRequested() = TabsPreviousRequestedEvent;

  /// Show tab [number] counted from 1; 9 is the last.
  const factory TabsEvent.numbered(int number) = TabsNumberedEvent;

  /// Go back ([back]) or forward in the showing tab's history alone. Sent by
  /// the navigation history.
  const factory TabsEvent.stepRequested({required bool back}) =
      TabsStepRequestedEvent;

  /// The screen showed the note the state's `pendingNoteId` named.
  const factory TabsEvent.consumed() = TabsConsumedEvent;

  /// The library changed these notes: titles and locks follow, and the tabs
  /// of notes trashed or deleted go.
  const factory TabsEvent.changesReceived(List<String> ids) =
      TabsChangesReceivedEvent;
}
