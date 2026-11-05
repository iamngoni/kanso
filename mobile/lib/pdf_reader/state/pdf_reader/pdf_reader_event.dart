// Kanso — pdf_reader_event.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'pdf_reader_bloc.dart';

@freezed
sealed class PdfReaderEvent with _$PdfReaderEvent {
  /// Opens attachment [file] of note [noteId] and shows it at [page]
  /// (1-based), with the highlight [highlight] in view when the link named
  /// one.
  const factory PdfReaderEvent.started({
    required String noteId,
    required String file,
    required String name,
    int? page,
    String? highlight,
    @Default(false) bool dark,
  }) = PdfReaderStartedEvent;

  /// The password of an encrypted PDF was typed: open it with [password].
  const factory PdfReaderEvent.passwordSubmitted(String password) =
      PdfReaderPasswordSubmittedEvent;

  /// The page could not be opened: open it again.
  const factory PdfReaderEvent.retried() = PdfReaderRetriedEvent;

  /// The pages are drawn at [zoom].
  const factory PdfReaderEvent.zoomChanged(PdfZoom zoom) =
      PdfReaderZoomChangedEvent;

  /// The pages are drawn for a dark screen, or back.
  const factory PdfReaderEvent.invertChanged({required bool invert}) =
      PdfReaderInvertChangedEvent;

  /// The app's theme changed: [dark] is whether it is now dark. A PDF kept
  /// as printed stays printed; the saved choice is not changed by this.
  const factory PdfReaderEvent.darkChanged({required bool dark}) =
      PdfReaderDarkChangedEvent;

  /// The page in view changed: [page] is 1-based.
  const factory PdfReaderEvent.pageChanged(int page) =
      PdfReaderPageChangedEvent;

  /// The page [page] is to be shown: the view scrolls to it.
  const factory PdfReaderEvent.jumpTo(int page) = PdfReaderJumpToEvent;

  /// A word was held at a point of [page] (1-based; [x] and [y] are
  /// fractions of the page): select it and the selection that follows.
  const factory PdfReaderEvent.wordSelected(int page, double x, double y) =
      PdfReaderWordSelectedEvent;

  /// A line was tapped twice at a point of [page]: select it.
  const factory PdfReaderEvent.lineSelected(int page, double x, double y) =
      PdfReaderLineSelectedEvent;

  /// A selection handle was dragged to a point of [page]; the end given by
  /// [fromStart] is fixed.
  const factory PdfReaderEvent.selectionExtended(
    int page,
    double x,
    double y, {
    required bool fromStart,
  }) = PdfReaderSelectionExtendedEvent;

  /// The selection was dismissed.
  const factory PdfReaderEvent.selectionCleared() =
      PdfReaderSelectionClearedEvent;

  /// The search text changed.
  const factory PdfReaderEvent.searchChanged(String query) =
      PdfReaderSearchChangedEvent;

  /// The search for [query] is run now (its text settled).
  const factory PdfReaderEvent.searchRun(String query) =
      PdfReaderSearchRunEvent;

  /// The next hit of the search is shown.
  const factory PdfReaderEvent.searchNext() = PdfReaderSearchNextEvent;

  /// The hit before the one shown is shown.
  const factory PdfReaderEvent.searchPrevious() = PdfReaderSearchPreviousEvent;

  /// The search bar was closed.
  const factory PdfReaderEvent.searchClosed() = PdfReaderSearchClosedEvent;

  /// The selection is highlighted in [color], its quote put into the note
  /// when [quote].
  const factory PdfReaderEvent.highlightRequested({
    required String color,
    required bool quote,
  }) = PdfReaderHighlightRequestedEvent;

  /// The highlight [id] is brought into view, when the link named it.
  const factory PdfReaderEvent.highlightFocused(String? id) =
      PdfReaderHighlightFocusedEvent;

  /// The notice was read.
  const factory PdfReaderEvent.noticeDismissed() =
      PdfReaderNoticeDismissedEvent;

  /// A lock closed the note while its PDF was open.
  const factory PdfReaderEvent.lockChanged() = PdfReaderLockChangedEvent;
}
