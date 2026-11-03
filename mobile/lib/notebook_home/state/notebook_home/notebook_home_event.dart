// Kanso — notebook_home_event.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'notebook_home_bloc.dart';

@freezed
sealed class NotebookHomeEvent with _$NotebookHomeEvent {
  /// The page of notebook [bookId] opened: read the words, then the page.
  const factory NotebookHomeEvent.started(String bookId) =
      NotebookHomeStartedEvent;

  /// Read the page again (a note was edited, the lock changed).
  const factory NotebookHomeEvent.reopened() = NotebookHomeReopenedEvent;

  /// Ask for the map; it is heavy, so it never holds the sections up.
  const factory NotebookHomeEvent.mapRequested() =
      NotebookHomeMapRequestedEvent;

  const factory NotebookHomeEvent.refreshAllRequested() =
      NotebookHomeRefreshAllRequestedEvent;

  const factory NotebookHomeEvent.sectionRefreshRequested(String key) =
      NotebookHomeSectionRefreshRequestedEvent;

  const factory NotebookHomeEvent.pinned(String key, {required bool on}) =
      NotebookHomePinnedEvent;

  const factory NotebookHomeEvent.hidden(String key, {required bool hidden}) =
      NotebookHomeHiddenEvent;

  const factory NotebookHomeEvent.hiddenShown() = NotebookHomeHiddenShownEvent;

  const factory NotebookHomeEvent.notRight(String key, int index) =
      NotebookHomeNotRightEvent;

  const factory NotebookHomeEvent.copiedIntoHome(String key) =
      NotebookHomeCopiedIntoHomeEvent;

  const factory NotebookHomeEvent.taskTicked(
    PageTask task, {
    required bool checked,
  }) = NotebookHomeTaskTickedEvent;

  const factory NotebookHomeEvent.allShown(String key) =
      NotebookHomeAllShownEvent;

  const factory NotebookHomeEvent.customised(
    List<String> order,
    List<String> hidden,
  ) = NotebookHomeCustomisedEvent;

  const factory NotebookHomeEvent.layoutReset() = NotebookHomeLayoutResetEvent;

  const factory NotebookHomeEvent.homeChosen(String noteId) =
      NotebookHomeHomeChosenEvent;

  const factory NotebookHomeEvent.newHomeRequested() =
      NotebookHomeNewHomeRequestedEvent;

  /// The page took the one-off news in the state (a copy, a new note).
  const factory NotebookHomeEvent.noticeTaken() = NotebookHomeNoticeTakenEvent;
}
