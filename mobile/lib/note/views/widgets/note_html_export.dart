// Kanso — note_html_export.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';

import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/services/preview_theme_fitter.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/preview_themes_of.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../embargo/models/data/embargo_gate.dart';
import '../../../embargo/views/widgets/embargo_sheet.dart';
import '../../../injection.dart';
import '../../../metrics/models/repos/abs/usage_metrics_repository.dart';
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
  final overlay = Overlay.of(context, rootOverlay: true);
  // Where the share sheet points on a tablet. Taken before the menu closes.
  final origin = exportOriginOf(context);
  // The themes the note's preview is drawn in, taken while the menu is open.
  final themes = previewThemesOf(context);
  // A note under embargo asks first.
  final go = await confirmEmbargo(
    context,
    noteId: noteId,
    gate: EmbargoGate.export,
  );
  if (!go || !context.mounted) {
    return;
  }
  final progress = OverlayEntry(
    builder: (_) => NoteExportProgress(label: l10n.noteExportPreparingPage),
  );
  overlay.insert(progress);
  try {
    final exporter = NoteHtmlExporter(
      getIt<RendererRepository>(),
      getIt<PreviewThemeFitter>(),
    );
    final written = await exporter.export(
      noteId: noteId,
      themes: themes,
    );
    progress.remove();
    switch (written) {
      case Right(value: final path):
        await shareHtml(path, path.split('/').last, origin: origin);
        unawaited(
          getIt<UsageMetricsRepository>().record(UsageFeature.export),
        );
      case Left():
        KansoFeedback.show(
          context,
          l10n.noteExportFailed,
          tone: KansoFeedbackTone.error,
        );
    }
  } on Object {
    progress.remove();
    KansoFeedback.show(
      context,
      l10n.noteExportFailed,
      tone: KansoFeedbackTone.error,
    );
  } finally {
    progress.remove();
  }
}
