// Kanso — renderer_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Drawing done by the Rust core: previews, source highlighting, themes,
/// math and diagrams.
abstract interface class RendererRepository {
  /// The note [id] as a standalone HTML page in [palette]'s colours; locked
  /// while the note is locked and closed.
  Future<Either<ApplicationError, String>> previewHtml(
    String id,
    PaletteDto palette,
  );

  /// The note [id] as the app's own preview page: links keep addresses that
  /// [linkTarget] understands, task boxes can be tapped, and query views are
  /// tables. Locked while the note is locked and closed.
  Future<Either<ApplicationError, String>> previewPage(
    String id,
    PaletteDto palette,
    PreviewOptions options,
  );

  /// What the address [url], tapped in the preview page of the note
  /// [noteId], leads to; anything unknown or unsafe is
  /// [PreviewLink.nothing].
  Future<Either<ApplicationError, PreviewLink>> linkTarget(
    String noteId,
    String url,
  );

  /// The text of the [index]-th code block of the note [noteId], for its
  /// copy button.
  Future<Either<ApplicationError, String>> codeBlock(String noteId, int index);

  /// The runs of [text] between [start] and [end] (UTF-16 units) that the
  /// Markdown highlighter styles.
  Future<Either<ApplicationError, List<SpanRow>>> highlight(
    String text, {
    required int start,
    required int end,
  });

  /// A theme file's JSON, fitted for drawing.
  Future<Either<ApplicationError, ThemeTokens>> themeTokens(String themeJson);

  /// The built-in themes in the desktop's order, each fitted as the desktop
  /// draws it.
  Future<Either<ApplicationError, List<BuiltinTheme>>> builtinThemes();

  /// The built-in theme [id] (`nord`), fitted.
  Future<Either<ApplicationError, ThemeTokens>> builtinThemeTokens(String id);

  /// The built-in icon themes in the desktop's order (Lucide, Kanso's own,
  /// first), each with a few icons that show its look.
  Future<Either<ApplicationError, List<IconThemeRow>>> iconThemes();

  /// Every icon the app draws as the built-in icon theme [id] draws it; an
  /// icon the theme leaves out has no SVG and is drawn as Lucide's.
  Future<Either<ApplicationError, List<ThemedIcon>>> iconThemeIcons(String id);

  /// The runs of [text] between [start] and [end] (UTF-16 units) that the
  /// built-in theme [themeId] styles; text between runs is plain.
  Future<Either<ApplicationError, List<StyledRun>>> styledRuns(
    String text, {
    required int start,
    required int end,
    required String themeId,
  });

  Future<Either<ApplicationError, SvgRow>> mathSvg(
    String source, {
    required bool display,
    required PaletteDto palette,
  });

  Future<Either<ApplicationError, SvgRow>> diagramSvg(
    DiagramKind kind,
    String source,
    PaletteDto palette,
  );

  /// Hands over the app's fonts so diagrams are set in them; whether this
  /// call took effect (only the first before a drawing does).
  Future<Either<ApplicationError, bool>> setFonts(List<Uint8List> fonts);
}
