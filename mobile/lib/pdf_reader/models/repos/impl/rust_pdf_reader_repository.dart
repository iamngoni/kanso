// Kanso — rust_pdf_reader_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/pdf.dart';
import '../../../../core/bridge/api/pdf_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../../data/pdf_opening.dart';
import '../abs/pdf_reader_repository.dart';

/// [PdfReaderRepository] over the Rust core's library: the reader runs the
/// desktop's PDF code in the app, on a thread of its own.
@LazySingleton(as: PdfReaderRepository)
class RustPdfReaderRepository implements PdfReaderRepository {
  RustPdfReaderRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, PdfOpening>> open({
    required String noteId,
    required String file,
    String? password,
  }) => attempt(() async {
    final result = await _library.pdf().open(
      noteId: noteId,
      file: file,
      password: password,
    );
    final declined = result.declined;
    if (declined != null) {
      return PdfOpening.declined(reason: declined);
    }
    final reader = result.reader;
    final info = result.info;
    if (reader == null || info == null) {
      // The bridge answers one or the other; a third answer is a bug.
      throw StateError(
        'The PDF reader answered with neither a document nor a reason.',
      );
    }
    return PdfOpening.opened(
      session: _RustPdfDocumentSession(reader),
      info: info,
    );
  });

  @override
  Future<Either<ApplicationError, PdfMarks>> marks({
    required String noteId,
    required String file,
  }) => attempt(() => _library.pdf().marks(noteId: noteId, file: file));
}

/// A document the Rust reader holds open. Every call goes to it; [dispose]
/// ends its worker thread and lets go of it.
class _RustPdfDocumentSession implements PdfDocumentSession {
  _RustPdfDocumentSession(this._reader);

  final KansoPdfReader _reader;
  bool _closed = false;

  @override
  Future<Either<ApplicationError, PdfPageImage>> render(
    int page,
    int widthPx, {
    required bool invert,
  }) => attempt(
    () => _reader.render(page: page, widthPx: widthPx, invert: invert),
  );

  @override
  Future<Either<ApplicationError, PdfSelection?>> wordAt(
    int page,
    double x,
    double y,
  ) => attempt(() => _reader.wordAt(page: page, x: x, y: y));

  @override
  Future<Either<ApplicationError, PdfSelection?>> lineAt(
    int page,
    double x,
    double y,
  ) => attempt(() => _reader.lineAt(page: page, x: x, y: y));

  @override
  Future<Either<ApplicationError, PdfSelection?>> extend(
    PdfCharPos anchor,
    int page,
    double x,
    double y,
  ) => attempt(() => _reader.extend(anchor: anchor, page: page, x: x, y: y));

  @override
  Future<Either<ApplicationError, PdfFound>> find(String query, int limit) =>
      attempt(() => _reader.find(query: query, limit: limit));

  @override
  Future<Either<ApplicationError, PdfEdited>> addHighlight({
    required String baseRev,
    required PdfCharPos start,
    required PdfCharPos end,
    required String color,
    required bool quote,
  }) => attempt(
    () => _reader.addHighlight(
      baseRev: baseRev,
      start: start,
      end: end,
      color: color,
      quote: quote,
    ),
  );

  @override
  void dispose() {
    if (_closed) {
      return;
    }
    _closed = true;
    _reader
      ..close()
      ..dispose();
  }
}
