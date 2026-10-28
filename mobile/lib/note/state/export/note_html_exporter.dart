// Kanso — note_html_exporter.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';
import 'dart:ui';

import 'package:dartz/dartz.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:share_plus/share_plus.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../preview/preview_bloc.dart';

/// The folder a page is written in before it is shared.
typedef HtmlFolder = Future<Directory> Function();

/// Writes a note as a web page: the page the desktop's "Export as web page"
/// saves, in the theme the note is drawn in. One self-contained file, with
/// its math, diagrams and drawings inline as SVG and its images as data
/// URIs, and no scripts.
///
/// The folder is a seam so the part that needs a device can be stood in for
/// in tests.
class NoteHtmlExporter {
  NoteHtmlExporter(this._renderer, {HtmlFolder? folder})
    : _folder = folder ?? getTemporaryDirectory;

  final RendererRepository _renderer;
  final HtmlFolder _folder;

  /// The note [noteId], drawn in [themeId], written as a web page.
  ///
  /// Answers the path it was written to, for [shareHtml]. A note that is
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
    final page = await _renderer.pageHtml(
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
      await File(path).writeAsString(drawn.html);
    } on Object catch (error) {
      return Left<ApplicationError, String>(
        ApplicationError.storage(message: '$error'),
      );
    }
    return Right<ApplicationError, String>(path);
  }
}

/// Offers the web page at [path] to the system's share sheet. [origin] is
/// where the sheet points on a tablet, which asks for one.
Future<void> shareHtml(String path, String name, {Rect? origin}) =>
    SharePlus.instance.share(
      ShareParams(
        files: <XFile>[XFile(path, mimeType: 'text/html', name: name)],
        sharePositionOrigin: origin,
      ),
    );
