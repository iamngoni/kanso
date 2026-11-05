// Kanso — pdf_highlights_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' hide State;
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../injection.dart';

/// Tells the "4 highlights" line in a note's preview which note it is in, so
/// a tap on it can open the list. Without one (a presentation, a widget
/// test) the line is only text.
class PdfNotesScope extends InheritedWidget {
  const PdfNotesScope({
    required this.noteId,
    required super.child,
    this.repository,
    super.key,
  });

  final String noteId;

  /// What reads and changes the highlights; the app's own when not given.
  final LibraryRepository? repository;

  static PdfNotesScope? maybeOf(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<PdfNotesScope>();

  /// Opens the list over the whole app.
  void open(BuildContext context) {
    PdfHighlightsSheet.show(context, noteId: noteId, repository: repository);
  }

  @override
  bool updateShouldNotify(PdfNotesScope oldWidget) =>
      noteId != oldWidget.noteId || repository != oldWidget.repository;
}

/// The highlights a note holds for its PDFs, as the desktop reader's panel
/// has them: each with its page, its words, its comment and whether its quote
/// is in the note. A tap opens a highlight to write a comment, change its
/// colour, put its quote back into the note or delete it. A tap can open the
/// PDF reader at the highlight or pencil-marked page.
class PdfHighlightsSheet extends StatefulWidget {
  const PdfHighlightsSheet({required this.noteId, this.repository, super.key});

  final String noteId;
  final LibraryRepository? repository;

  static Future<void> show(
    BuildContext context, {
    required String noteId,
    LibraryRepository? repository,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) =>
          PdfHighlightsSheet(noteId: noteId, repository: repository),
    );
  }

  @override
  State<PdfHighlightsSheet> createState() => _PdfHighlightsSheetState();
}

class _PdfHighlightsSheetState extends State<PdfHighlightsSheet> {
  late final LibraryRepository _library =
      widget.repository ?? getIt<LibraryRepository>();
  final TextEditingController _comment = TextEditingController();

  String _rev = '';
  List<PdfNotes>? _notes;
  ApplicationError? _error;
  String? _open;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  @override
  void dispose() {
    _comment.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    final note = await _library.note(widget.noteId);
    final notes = await _library.pdfHighlights(widget.noteId);
    if (!mounted) {
      return;
    }
    setState(() {
      note.fold((error) => _error = error, (view) => _rev = view.rev);
      notes.fold((error) => _error = error, (list) {
        _notes = list;
        _error = null;
      });
    });
  }

  Future<void> _change(
    PdfNotes pdf,
    PdfHighlightRow row,
    PdfHighlightChange change,
  ) async {
    setState(() => _busy = true);
    final Either<ApplicationError, Saved> done = await _library
        .pdfHighlightChange(
          noteId: widget.noteId,
          baseRev: _rev,
          file: pdf.file,
          id: row.id,
          change: change,
        );
    if (!mounted) {
      return;
    }
    await done.fold(
      (error) async {
        setState(() => _error = error);
        // The note moved on: read it again so the next change can be made.
        await _load();
      },
      (saved) async {
        _rev = saved.rev;
        await _load();
      },
    );
    if (mounted) {
      setState(() => _busy = false);
    }
  }

  void _toggle(PdfHighlightRow row) {
    setState(() {
      if (_open == row.id) {
        _open = null;
      } else {
        _open = row.id;
        _comment.text = row.comment ?? '';
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final notes = _notes;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            l10n.pdfNotesTitle,
            style: context.typography.barTitle,
          ),
        ),
        if (_error != null)
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: KansoInlineMessage(message: _error!.message),
          ),
        if (notes == null && _error == null)
          const SizedBox(
            height: KansoSpacing.xl,
            child: Center(child: KansoSpinner()),
          ),
        if (notes != null)
          Flexible(
            child: ListView(
              shrinkWrap: true,
              padding: const EdgeInsets.only(bottom: KansoSpacing.md),
              children: <Widget>[
                if (notes.every(
                  (pdf) => pdf.highlights.isEmpty && pdf.ink.isEmpty,
                ))
                  Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: Text(
                      l10n.pdfNotesEmpty,
                      style: context.typography.secondary,
                    ),
                  ),
                for (final pdf in notes) ..._pdf(context, pdf),
              ],
            ),
          ),
      ],
    );
  }

  List<Widget> _pdf(BuildContext context, PdfNotes pdf) {
    if (pdf.highlights.isEmpty && pdf.ink.isEmpty && pdf.why == null) {
      return const <Widget>[];
    }
    final l10n = context.l10n;
    return <Widget>[
      Padding(
        padding: const EdgeInsetsDirectional.fromSTEB(
          KansoSpacing.md,
          KansoSpacing.sm,
          KansoSpacing.md,
          KansoSpacing.xs,
        ),
        child: Text(pdf.name, style: context.typography.sectionLabel),
      ),
      if (!pdf.editable)
        Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xs,
          ),
          child: Text(
            l10n.pdfNotesReadOnly,
            key: const ValueKey<String>('pdf-notes-read-only'),
            style: context.typography.secondary,
          ),
        ),
      for (final row in pdf.highlights) _row(context, pdf, row),
      if (pdf.ink.isNotEmpty) ...<Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            l10n.pdfNotesInkTitle,
            key: ValueKey<String>('pdf-ink-title-${pdf.file}'),
            style: context.typography.sectionLabel,
          ),
        ),
        for (final row in pdf.ink) _inkRow(context, pdf, row),
      ],
    ];
  }

  /// One page with pencil marks. Drawing happens in the PDF reader; the sheet
  /// can also clear all marks from this page.
  Widget _inkRow(BuildContext context, PdfNotes pdf, PdfInkRow row) {
    final l10n = context.l10n;
    return Container(
      key: ValueKey<String>('pdf-ink-row-${pdf.file}-${row.page}'),
      margin: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.sm,
        vertical: 2,
      ),
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.sm,
        vertical: KansoSpacing.xs,
      ),
      child: Row(
        children: <Widget>[
          Expanded(
            child: Text(
              '${l10n.pdfNotesPage(row.page)} · '
              '${l10n.pdfNotesInkStrokes(row.strokes)}',
              style: context.typography.body,
            ),
          ),
          if (pdf.editable)
            KansoButton(
              key: ValueKey<String>('pdf-ink-clear-${pdf.file}-${row.page}'),
              label: l10n.pdfNotesInkClear,
              compact: true,
              expanded: false,
              variant: KansoButtonVariant.secondary,
              onPressed: _busy ? null : () => _clearInk(pdf, row),
            ),
        ],
      ),
    );
  }

  Future<void> _clearInk(PdfNotes pdf, PdfInkRow row) async {
    setState(() => _busy = true);
    final Either<ApplicationError, Saved> done = await _library.pdfInkClear(
      noteId: widget.noteId,
      baseRev: _rev,
      file: pdf.file,
      page: row.page,
    );
    if (!mounted) {
      return;
    }
    await done.fold(
      (error) async {
        setState(() => _error = error);
        await _load();
      },
      (saved) async {
        _rev = saved.rev;
        await _load();
      },
    );
    if (mounted) {
      setState(() => _busy = false);
    }
  }

  Widget _row(BuildContext context, PdfNotes pdf, PdfHighlightRow row) {
    final l10n = context.l10n;
    final colors = context.colors;
    final open = _open == row.id;
    return Container(
      key: ValueKey<String>('pdf-row-${row.id}'),
      margin: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.sm,
        vertical: 2,
      ),
      decoration: BoxDecoration(
        color: open ? colors.fill : null,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoPressable(
            onTap: () => _toggle(row),
            semanticLabel: l10n.pdfNotesPage(row.page),
            child: Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Row(
                    children: <Widget>[
                      Container(
                        width: 10,
                        height: 10,
                        decoration: BoxDecoration(
                          color: highlightColor(row.color),
                          shape: BoxShape.circle,
                        ),
                      ),
                      const SizedBox(width: KansoSpacing.xs),
                      Text(
                        l10n.pdfNotesPage(row.page),
                        style: context.typography.secondary,
                      ),
                      if (!row.inNote) ...<Widget>[
                        const SizedBox(width: KansoSpacing.xs),
                        Text(
                          '· ${l10n.pdfNotesNotInNote}',
                          key: ValueKey<String>('pdf-not-in-note-${row.id}'),
                          style: context.typography.secondary,
                        ),
                      ],
                    ],
                  ),
                  const SizedBox(height: 2),
                  Text(
                    row.text,
                    maxLines: open ? null : 3,
                    overflow: open ? null : TextOverflow.ellipsis,
                    style: context.typography.body,
                  ),
                  if (row.comment != null && !open) ...<Widget>[
                    const SizedBox(height: 2),
                    Text(
                      row.comment!,
                      key: ValueKey<String>('pdf-comment-${row.id}'),
                      style: context.typography.secondary,
                    ),
                  ],
                ],
              ),
            ),
          ),
          if (open && pdf.editable) _actions(context, pdf, row),
        ],
      ),
    );
  }

  Widget _actions(BuildContext context, PdfNotes pdf, PdfHighlightRow row) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.sm,
        0,
        KansoSpacing.sm,
        KansoSpacing.sm,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoTextField(
            controller: _comment,
            hint: l10n.pdfNotesCommentHint,
            maxLines: 4,
            autocorrect: true,
            semanticLabel: l10n.pdfNotesCommentHint,
          ),
          const SizedBox(height: KansoSpacing.xs),
          Row(
            children: <Widget>[
              for (final color in const <String>[
                'yellow',
                'green',
                'blue',
                'pink',
              ])
                Padding(
                  padding: const EdgeInsetsDirectional.only(end: 10),
                  child: KansoPressable(
                    onTap: _busy
                        ? null
                        : () => _change(
                            pdf,
                            row,
                            PdfHighlightChange.color(color: color),
                          ),
                    semanticLabel: l10n.pdfNotesColor(color),
                    child: Container(
                      key: ValueKey<String>('pdf-color-$color'),
                      width: 28,
                      height: 28,
                      decoration: BoxDecoration(
                        color: highlightColor(color),
                        shape: BoxShape.circle,
                        border: Border.all(
                          width: 2,
                          color: row.color == color
                              ? context.colors.text
                              : context.colors.divider,
                        ),
                      ),
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.pdfNotesSaveNote,
            compact: true,
            onPressed: _busy
                ? null
                : () => _change(
                    pdf,
                    row,
                    PdfHighlightChange.comment(comment: _comment.text),
                  ),
          ),
          if (!row.inNote) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: l10n.pdfNotesAddToNote,
              compact: true,
              variant: KansoButtonVariant.secondary,
              onPressed: _busy
                  ? null
                  : () => _change(
                      pdf,
                      row,
                      const PdfHighlightChange.addQuote(),
                    ),
            ),
          ],
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.pdfNotesDelete,
            compact: true,
            variant: KansoButtonVariant.danger,
            onPressed: _busy
                ? null
                : () => _change(pdf, row, const PdfHighlightChange.delete()),
          ),
        ],
      ),
    );
  }
}

/// The colour a highlight is drawn in: the desktop's four.
Color highlightColor(String name) => switch (name) {
  'green' => const Color(0xFF34C759),
  'blue' => const Color(0xFF3B9CFF),
  'pink' => const Color(0xFFFF5C9A),
  _ => const Color(0xFFFFD60A),
};
