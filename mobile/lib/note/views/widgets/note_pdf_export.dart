// Kanso — note_pdf_export.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/syntax_theme_id.dart';
import '../../../injection.dart';
import '../../state/export/note_pdf_exporter.dart';

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
  final origin = _originOf(context);
  final progress = OverlayEntry(builder: (_) => const _ExportProgress());
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

/// The part of the screen a sheet points at, from [context]'s box.
Rect? _originOf(BuildContext context) {
  final box = context.findRenderObject();
  if (box is! RenderBox || !box.hasSize) {
    return null;
  }
  return box.localToGlobal(Offset.zero) & box.size;
}

/// What is shown while the page is written: nothing can be tapped.
class _ExportProgress extends StatelessWidget {
  const _ExportProgress();

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: <Widget>[
        const ModalBarrier(dismissible: false, color: Color(0x33000000)),
        Center(
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.lg,
              vertical: KansoSpacing.md,
            ),
            decoration: BoxDecoration(
              color: context.colors.elevated,
              borderRadius: KansoRadii.mediumBorder,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Text(
                  context.l10n.noteExportPreparing,
                  style: context.typography.body,
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
