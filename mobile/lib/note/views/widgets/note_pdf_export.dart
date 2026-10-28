// Kanso — note_pdf_export.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';

import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../injection.dart';
import '../../state/export/note_pdf_exporter.dart';
import 'note_export_progress.dart';

/// Exports the note [noteId] as a PDF from its menu: a spinner while the
/// page is written, then the system's share sheet, or a word about what went
/// wrong.
Future<void> exportNotePdf(
  BuildContext context, {
  required String noteId,
}) async {
  final l10n = context.l10n;
  final messenger = ScaffoldMessenger.maybeOf(context);
  final overlay = Overlay.of(context, rootOverlay: true);
  // Where the share sheet points on a tablet. Taken before the menu closes.
  final origin = exportOriginOf(context);
  final progress = OverlayEntry(
    builder: (_) => NoteExportProgress(label: l10n.noteExportPreparing),
  );
  overlay.insert(progress);
  try {
    final exporter = NotePdfExporter(getIt<RendererRepository>());
    final written = await exporter.export(
      noteId: noteId,
      themeId: syntaxThemeId(context),
    );
    progress.remove();
    switch (written) {
      case Right(value: final path):
        await sharePdf(path, path.split('/').last, origin: origin);
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
