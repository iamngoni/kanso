// Kanso — make_flashcards_sheet.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/ai.dart';
import '../../../core/bridge/api/cards.dart';
import '../../../core/bridge/api/cards_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../core/views/widgets/kanso_bottom_sheet.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';

/// Opens **Make Flashcards…** for note [noteId]: the model suggests cards
/// from that note alone, the person ticks the ones to keep, and they are
/// written into the note under `## Flashcards` in one edit.
Future<void> showMakeFlashcards(BuildContext context, String noteId) {
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => KansoBottomSheet(
      child: MakeFlashcardsSheet(noteId: noteId),
    ),
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

  @override
  State<MakeFlashcardsSheet> createState() => _MakeFlashcardsSheetState();
}

class _MakeFlashcardsSheetState extends State<MakeFlashcardsSheet> {
  late final KansoCards _cards = getIt<KansoLibrary>().cards();
  late final KansoAiRun _run = getIt<KansoAi>().newRun();
  StreamSubscription<SuggestEvent>? _events;
  List<_Row> _rows = <_Row>[];
  String _status = FlashcardsText.makeReading;
  String? _error;
  bool _working = true;
  bool _adding = false;

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
              _error = FlashcardsText.noSuggestions;
            }
          });
        case SuggestEvent_Failed(:final message):
          setState(() {
            _error = message;
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
    super.dispose();
  }

  Future<void> _edit(_Row row) async {
    final q = TextEditingController(text: row.card.question);
    final a = TextEditingController(text: row.card.answer);
    final saved = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            TextField(
              controller: q,
              decoration: const InputDecoration(labelText: 'Question'),
              maxLines: null,
            ),
            TextField(
              controller: a,
              decoration: const InputDecoration(labelText: 'Answer'),
              maxLines: null,
            ),
          ],
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Done'),
          ),
        ],
      ),
    );
    if ((saved ?? false) &&
        q.text.trim().isNotEmpty &&
        a.text.trim().isNotEmpty) {
      setState(() {
        row.card = SuggestedCard(
          question: q.text.trim(),
          answer: a.text.trim(),
          source: row.card.source,
          line: row.card.line,
          sourceMissing: row.card.sourceMissing,
        );
      });
    }
    q.dispose();
    a.dispose();
  }

  Future<void> _add() async {
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
          SnackBar(content: Text(FlashcardsText.added(n))),
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
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Text(FlashcardsText.makeCards, style: type.heading2),
        const SizedBox(height: KansoSpacing.sm),
        if (_working)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: KansoSpacing.lg),
            child: Row(
              children: <Widget>[
                const SizedBox(
                  width: 18,
                  height: 18,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Text(_status, style: type.secondary),
              ],
            ),
          ),
        if (_error case final message?)
          Padding(
            padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
            child: Text(
              message,
              style: type.secondary.copyWith(color: colors.danger),
            ),
          ),
        Flexible(
          child: ListView(
            shrinkWrap: true,
            children: <Widget>[
              for (final row in _rows)
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  controlAffinity: ListTileControlAffinity.leading,
                  value: row.on,
                  onChanged: (v) => setState(() => row.on = v ?? false),
                  title: Text(
                    '${row.card.question} → ${row.card.answer}',
                    style: type.body,
                  ),
                  subtitle: row.card.sourceMissing
                      ? Text(
                          FlashcardsText.sourceNotFound,
                          style: type.meta.copyWith(color: colors.warning),
                        )
                      : row.card.line == null
                      ? null
                      : Text(
                          'from line ${row.card.line! + 1}',
                          style: type.meta.copyWith(
                            color: colors.textSecondary,
                          ),
                        ),
                  secondary: IconButton(
                    icon: const Icon(Icons.edit_outlined),
                    onPressed: () => _edit(row),
                  ),
                ),
            ],
          ),
        ),
        const SizedBox(height: KansoSpacing.sm),
        KansoButton(
          label: FlashcardsText.addCards(ticked),
          onPressed: ticked == 0 || _adding || _working ? null : _add,
        ),
      ],
    );
  }
}
