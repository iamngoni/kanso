// Kanso — note_html_export.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';

import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../injection.dart';
import '../../state/export/note_html_exporter.dart';
import 'note_export_progress.dart';

/// Exports the note [noteId] as a web page from its menu: a spinner while
/// the page is written, then the system's share sheet, or a word about what
/// went wrong.
Future<void> exportNoteHtml(
  BuildContext context, {
  required String noteId,
}) async {
  final l10n = context.l10n;
  final messenger = ScaffoldMessenger.maybeOf(context);
  final overlay = Overlay.of(context, rootOverlay: true);
  // Where the share sheet points on a tablet. Taken before the menu closes.
  final origin = exportOriginOf(context);
  final progress = OverlayEntry(
    builder: (_) => NoteExportProgress(label: l10n.noteExportPreparingPage),
  );
  overlay.insert(progress);
  try {
    final exporter = NoteHtmlExporter(getIt<RendererRepository>());
    final written = await exporter.export(
      noteId: noteId,
      themeId: syntaxThemeId(context),
    );
    progress.remove();
    switch (written) {
      case Right(value: final path):
        await shareHtml(path, path.split('/').last, origin: origin);
      case Left():
        messenger?.showSnackBar(
          SnackBar(content: Text(l10n.noteExportFailed)),
        );
    }
  } on Object {
    progress.remove();
    messenger?.showSnackBar(SnackBar(content: Text(l10n.noteExportFailed)));
  } finally {
    progress.remove();
  }
}
