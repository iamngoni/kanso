// Kanso — pdf_reader_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/pdf_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../data/pdf_opening.dart';

/// A PDF open on the phone (issue #43): the Rust core's reader, seen from
/// the app. Pages, the text layer and highlights are the desktop reader's
/// code (`kanso-pdf`), run inside the app; every call is answered off the
/// UI thread.
///
/// [dispose] ends the reader and frees its page cache.
abstract interface class PdfDocumentSession {
  /// Page [page] (0-based) as BGRA pixels [widthPx] wide, for a dark reader
  /// when [invert]: text light, pictures in their own colours.
  Future<Either<ApplicationError, PdfPageImage>> render(
    int page,
    int widthPx, {
    required bool invert,
  });

  /// The word at a press ([page] 1-based; [x] and [y] fractions of the page
  /// from its top left), when one is near.
  Future<Either<ApplicationError, PdfSelection?>> wordAt(
    int page,
    double x,
    double y,
  );

  /// The line of text at the press.
  Future<Either<ApplicationError, PdfSelection?>> lineAt(
    int page,
    double x,
    double y,
  );

  /// The selection from the fixed end [anchor] to the character nearest the
  /// point: a handle was dragged there.
  Future<Either<ApplicationError, PdfSelection?>> extend(
    PdfCharPos anchor,
    int page,
    double x,
    double y,
  );

  /// Every hit of [query] (the first [limit] of them in full).
  Future<Either<ApplicationError, PdfFound>> find(String query, int limit);

  /// Highlights a selection in [color]; with [quote] its quote goes into the
  /// note in the same save. A note that moved meanwhile has the highlight
  /// made again on its newer text, never dropped.
  Future<Either<ApplicationError, PdfEdited>> addHighlight({
    required String baseRev,
    required PdfCharPos start,
    required PdfCharPos end,
    required String color,
    required bool quote,
  });

  /// Saves a pencil stroke on [page], in 1000-unit PDF page coordinates.
  Future<Either<ApplicationError, PdfEdited>> addInkStroke({
    required String baseRev,
    required int page,
    required PdfInkStroke stroke,
  });

  void dispose();
}

/// Opening a PDF attachment of a note, and the highlights it has there.
abstract interface class PdfReaderRepository {
  /// Opens attachment [file] of the note [noteId]. The [password] is for
  /// this call only. Answers are not errors: a PDF that needs a password, a
  /// locked note and a refused file come back as [PdfOpening.declined].
  Future<Either<ApplicationError, PdfOpening>> open({
    required String noteId,
    required String file,
    String? password,
  });

  /// The note's highlights for [file], as the reader draws them, with the
  /// revision they were read at.
  Future<Either<ApplicationError, PdfMarks>> marks({
    required String noteId,
    required String file,
  });
}
