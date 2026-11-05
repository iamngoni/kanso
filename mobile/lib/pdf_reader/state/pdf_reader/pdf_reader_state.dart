// Kanso — pdf_reader_state.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'pdf_reader_bloc.dart';

/// Where opening the PDF stands.
enum PdfReaderStatus {
  /// The document is being read.
  opening,

  /// The pages are drawn.
  opened,

  /// The document answered why it will not open (a password, a locked note).
  declined,

  /// Opening it failed.
  failed,
}

@freezed
abstract class PdfReaderState with _$PdfReaderState {
  const factory PdfReaderState({
    /// The note the attachment belongs to.
    @Default('') String noteId,

    /// The attachment (`file:<id>`).
    @Default('') String file,

    /// The file's name, shown in the bar.
    @Default('') String name,

    /// Where opening stands.
    @Default(PdfReaderStatus.opening) PdfReaderStatus status,

    /// What opened: its pages, their sizes and whether highlights can be
    /// written.
    PdfInfo? info,

    /// The note's highlights for the document, with the revision they were
    /// read at.
    PdfMarks? marks,

    /// Why it was not opened, when the document itself said so.
    PdfDeclined? declined,

    /// Why opening it failed, when it did.
    ApplicationError? error,

    /// Why the highlights could not be read; the pages are still readable.
    ApplicationError? marksError,

    /// The page width, as a multiple of the screen's.
    @Default(PdfZoom.fitWidth) PdfZoom zoom,

    /// Whether pages are drawn for a dark screen.
    @Default(false) bool invert,

    /// The page in view, 1-based.
    @Default(1) int page,

    /// Counts the times the page was asked for (a search's hit, a highlight,
    /// a typed page): the view scrolls when it changes, not when a scroll
    /// itself moved [page].
    @Default(0) int jump,

    /// The selection, when one is made.
    PdfSelection? selection,

    /// The text searched for.
    @Default('') String query,

    /// The hits of [query]; null until a search has run.
    PdfFound? found,

    /// Which hit of [found] is shown, 0-based.
    @Default(0) int hit,

    /// Whether a selection is being highlighted.
    @Default(false) bool adding,

    /// Whether the password last typed was wrong.
    @Default(false) bool passwordWrong,

    /// The highlight a link named, to keep in view.
    String? focusHighlight,

    /// Why the last action did nothing, said once and dismissed.
    ApplicationError? notice,
  }) = _PdfReaderState;
}
