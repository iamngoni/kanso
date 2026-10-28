// Kanso — note_pdf_exporter.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:ui';

import 'package:dartz/dartz.dart';
import 'package:kanso_print/kanso_print.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../preview/preview_bloc.dart';

/// The folder a PDF is written in before it is shared.
typedef PdfFolder = Future<Directory> Function();

/// How a page becomes a PDF at [path]: the plugin's native writer.
typedef PdfPrinter =
    Future<int> Function({
      required String html,
      required String path,
      required KansoPaper paper,
    });

/// The country a locale name names: `en_GB` and `en-GB` are both GB, and a
/// name without a region names none.
String? countryOf(String locale) {
  final parts = locale.split(RegExp('[_-]'));
  if (parts.length < 2) {
    return null;
  }
  final region = parts.last.trim();
  return region.length == 2 ? region.toUpperCase() : null;
}

/// Writes a note as a PDF: the page the desktop exports, printed in the
/// theme the note is drawn in, on A4 or Letter by the phone's country.
///
/// The seams are there so the parts that need a device (a temporary folder
/// and the native writer) can be stood in for in tests.
class NotePdfExporter {
  NotePdfExporter(
    this._renderer, {
    PdfFolder? folder,
    PdfPrinter? print,
    String? locale,
  }) : _folder = folder ?? getTemporaryDirectory,
       _print = print ?? _printPage,
       _countryCode = countryOf(locale ?? Platform.localeName);

  final RendererRepository _renderer;
  final PdfFolder _folder;
  final PdfPrinter _print;
  final String? _countryCode;

  /// The paper the phone's country prints on.
  KansoPaper get paper => paperFor(_countryCode);

  /// The note [noteId], drawn in [themeId], written as a PDF.
  ///
  /// Answers the path it was written to, for [`sharePdf`]. A note that is
  /// locked and closed comes back as [LockedError].
  Future<Either<ApplicationError, String>> export({
    required String noteId,
    required String themeId,
  }) async {
    final tokens = await _renderer.builtinThemeTokens(themeId);
    if (tokens case Left(value: final error)) {
      return Left<ApplicationError, String>(error);
    }
    final theme = tokens.getOrElse(() => throw StateError('unreachable'));
    final page = await _renderer.pagePdf(
      noteId,
      theme.palette,
      PreviewOptions(
        size: previewTextSize,
        // A page has no due chips: only the app's preview words them by day.
        utcOffsetSecs: 0,
        code: theme.syntax,
      ),
    );
    if (page case Left(value: final error)) {
      return Left<ApplicationError, String>(error);
    }
    final drawn = page.getOrElse(() => throw StateError('unreachable'));
    final folder = await _folder();
    final path = p.join(folder.path, drawn.fileName);
    try {
      await _print(html: drawn.html, path: path, paper: paper);
    } on Object catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.storage(message: '$error'),
      );
    }
    return Right<ApplicationError, String>(path);
  }
}

/// Offers the PDF at [path] to the system's share sheet. [origin] is where
/// the sheet points on a tablet, which asks for one.
Future<void> sharePdf(String path, String name, {Rect? origin}) =>
    SharePlus.instance.share(
      ShareParams(
        files: <XFile>[XFile(path, mimeType: 'application/pdf', name: name)],
        sharePositionOrigin: origin,
      ),
    );

/// Writes [html] at [path] with the plugin's native writer.
Future<int> _printPage({
  required String html,
  required String path,
  required KansoPaper paper,
}) => KansoPrint.htmlToPdf(html: html, outputPath: path, paper: paper);
