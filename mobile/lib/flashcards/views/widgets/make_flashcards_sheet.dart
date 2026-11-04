// Kanso — make_flashcards_sheet.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/ai.dart';
import '../../../core/bridge/api/cards.dart';
import '../../../core/bridge/api/cards_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_card.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';

/// Opens **Make Flashcards…** for note [noteId]: the model suggests cards
/// from that note alone, the person ticks the ones to keep, and they are
/// written into the note under `## Flashcards` in one edit.
Future<void> showMakeFlashcards(BuildContext context, String noteId) {
  return showKansoBottomSheet<void>(
    context,
    restingFraction: MakeFlashcardsSheet.restingFraction,
    builder: (context) => MakeFlashcardsSheet(noteId: noteId),
  );
}

class _Row {
  _Row(this.card) : on = !card.sourceMissing;

  SuggestedCard card;
  bool on;
}

class MakeFlashcardsSheet extends StatefulWidget {
  const MakeFlashcardsSheet({required this.noteId, super.key});

  final String noteId;

  /// How tall it opens; its handle pulls it up to the full height.
  static const double restingFraction = 0.78;

  @override
  State<MakeFlashcardsSheet> createState() => _MakeFlashcardsSheetState();
}

class _MakeFlashcardsSheetState extends State<MakeFlashcardsSheet> {
  late final KansoCards _cards = getIt<KansoLibrary>().cards();
  late final KansoAiRun _run = getIt<KansoAi>().newRun();
  StreamSubscription<SuggestEvent>? _events;
  List<_Row> _rows = <_Row>[];
  String _status = '';
  String? _error;
  bool _working = true;
  bool _adding = false;
  final TextEditingController _question = TextEditingController();
  final TextEditingController _answer = TextEditingController();
  _Row? _editing;

  @override
  void initState() {
    super.initState();
    _events = _cards.suggest(noteId: widget.noteId, run: _run).listen((e) {
      if (!mounted) {
        return;
      }
      switch (e) {
        case SuggestEvent_Status(:final line):
          setState(() => _status = line);
        case SuggestEvent_Done(:final cards):
          setState(() {
            _rows = cards.map(_Row.new).toList();
            _working = false;
            if (cards.isEmpty) {
              _error = context.l10n.flashcardsNoSuggestions;
            }
          });
        case SuggestEvent_Failed(:final message):
          setState(() {
            _error = FlashcardsText.failure(message, context.l10n);
            _working = false;
          });
        case SuggestEvent_Stopped():
          setState(() => _working = false);
      }
    });
  }

  @override
  void dispose() {
    _run.stop();
    unawaited(_events?.cancel());
    _question.dispose();
    _answer.dispose();
    super.dispose();
  }

  /// Opens card [row] for editing in place, keeping what was typed in
  /// another.
  void _edit(_Row row) {
    _finishEdit();
    _question.text = row.card.question;
    _answer.text = row.card.answer;
    setState(() => _editing = row);
  }

  /// Keeps the edit in progress, unless it left the question or the answer
  /// empty.
  void _finishEdit() {
    final row = _editing;
    if (row == null) {
      return;
    }
    final q = _question.text.trim();
    final a = _answer.text.trim();
    setState(() {
      _editing = null;
      if (q.isNotEmpty && a.isNotEmpty) {
        row.card = SuggestedCard(
          question: q,
          answer: a,
          source: row.card.source,
          line: row.card.line,
          sourceMissing: row.card.sourceMissing,
        );
      }
    });
  }

  Future<void> _add() async {
    _finishEdit();
    final picked = _rows.where((r) => r.on).toList();
    setState(() => _adding = true);
    try {
      final n = await _cards.addCards(
        noteId: widget.noteId,
        cards: <CardPair>[
          for (final r in picked)
            CardPair(question: r.card.question, answer: r.card.answer),
        ],
      );
      if (mounted) {
        Navigator.of(context).pop();
        ScaffoldMessenger.maybeOf(context)?.showSnackBar(
          SnackBar(content: Text(context.l10n.flashcardsAdded(n))),
        );
      }
    } on Object catch (e) {
      if (mounted) {
        setState(() {
          _error = applicationErrorOf(e).message;
          _adding = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final ticked = _rows.where((r) => r.on).length;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        SheetHeader(
          title: context.l10n.flashcardsMakeCards,
          actionLabel: context.l10n.settingsClose,
        ),
        if (_working) ...<Widget>[
          const KansoProgressBar(),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: Text(
              _status.isEmpty ? context.l10n.flashcardsMakeReading : _status,
              style: type.secondary.copyWith(color: colors.textSecondary),
            ),
          ),
        ],
        if (_error case final message?)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              0,
            ),
            child: KansoInlineMessage(message: message),
          ),
        Expanded(
          child: ListView(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.xs,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            children: <Widget>[
              for (final row in _rows)
                Padding(
                  padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
                  child: identical(row, _editing)
                      ? _editor(context)
                      : _card(context, row),
                ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: KansoButton(
            label: context.l10n.flashcardsAddCards(ticked),
            onPressed: ticked == 0 || _adding || _working ? null : _add,
          ),
        ),
      ],
    );
  }

  Widget _card(BuildContext context, _Row row) {
    final colors = context.colors;
    final type = context.typography;
    return KansoCard(
      padding: const EdgeInsets.all(KansoSpacing.sm),
      borderColor: row.on ? colors.accent : null,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.only(top: 2),
            child: KansoCheckbox(
              value: row.on,
              onChanged: (v) => setState(() => row.on = v),
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: GestureDetector(
              behavior: HitTestBehavior.opaque,
              onTap: () => setState(() => row.on = !row.on),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(row.card.question, style: type.body),
                  const SizedBox(height: KansoSpacing.xxs),
                  Text(
                    row.card.answer,
                    style: type.secondary.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                  if (row.card.sourceMissing)
                    Padding(
                      padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                      child: Text(
                        context.l10n.flashcardsSourceNotFound,
                        style: type.meta.copyWith(color: colors.warning),
                      ),
                    )
                  else if (row.card.line != null)
                    Padding(
                      padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                      child: Text(
                        'from line ${row.card.line! + 1}',
                        style: type.meta.copyWith(color: colors.textSecondary),
                      ),
                    ),
                ],
              ),
            ),
          ),
          KansoIconButton(
            icon: KansoIcons.pencil,
            semanticLabel: 'Edit card',
            onPressed: () => _edit(row),
          ),
        ],
      ),
    );
  }

  Widget _editor(BuildContext context) {
    final colors = context.colors;
    return KansoCard(
      padding: const EdgeInsets.all(KansoSpacing.sm),
      borderColor: colors.accent,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoTextField(
            controller: _question,
            hint: 'Question',
            autofocus: true,
            maxLines: 4,
          ),
          const Divider(height: KansoSpacing.md),
          KansoTextField(
            controller: _answer,
            hint: 'Answer',
            maxLines: 6,
          ),
          const SizedBox(height: KansoSpacing.xs),
          Align(
            alignment: AlignmentDirectional.centerEnd,
            child: KansoButton(
              label: context.l10n.noteListDone,
              compact: true,
              expanded: false,
              onPressed: _finishEdit,
            ),
          ),
        ],
      ),
    );
  }
}
