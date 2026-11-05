// Kanso — preview_pdf_notes.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_pressable.dart';
import '../pdf_highlights_sheet.dart';
import 'preview_style.dart';

/// The quiet line that stands for a PDF's highlights in a note: "4
/// highlights · 1 comment", in place of the data block the desktop reader
/// keeps in the note. A tap opens the list (where the note is known).
class PreviewPdfNotes extends StatelessWidget {
  const PreviewPdfNotes({required this.summary, super.key});

  final String summary;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final line = Text(
      summary,
      style: style.sized(0.8667).copyWith(color: style.muted),
    );
    final scope = PdfNotesScope.maybeOf(context);
    return Align(
      alignment: Alignment.centerLeft,
      child: scope == null
          ? line
          : KansoPressable(
              onTap: () => scope.open(context),
              semanticLabel: context.l10n.pdfNotesOpen(summary),
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 6),
                child: line,
              ),
            ),
    );
  }
}
