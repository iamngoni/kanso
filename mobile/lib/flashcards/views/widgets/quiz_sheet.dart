// Kanso — quiz_sheet.dart
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
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_card.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';

/// Opens **Quiz Me** for [noteIds] in a sheet over the note.
Future<void> showQuizMe(BuildContext context, List<String> noteIds) {
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => QuizSheet(noteIds: noteIds),
  );
}

/// Quiz Me: five questions from [noteIds], answered in words and graded by
/// the model against the note the question came from. Nothing is stored and
/// the review schedule is not touched; **Save as Card** turns a question
/// into a card in its note.
class QuizSheet extends StatefulWidget {
  const QuizSheet({required this.noteIds, super.key});

  final List<String> noteIds;

  @override
  State<QuizSheet> createState() => _QuizSheetState();
}

class _QuizSheetState extends State<QuizSheet> {
  late final KansoCards _cards = getIt<KansoLibrary>().cards();
  final TextEditingController _answer = TextEditingController();
  KansoAiRun? _run;
  StreamSubscription<Object?>? _events;
  List<QuizItem> _items = <QuizItem>[];
  int _at = 0;
  bool _loading = true;
  bool _grading = false;
  GradeEvent_Graded? _grade;
  String? _error;
  String? _saved;

  @override
  void initState() {
    super.initState();
    _start();
  }

  void _start() {
    final run = getIt<KansoAi>().newRun();
    _run = run;
    setState(() {
      _loading = true;
      _error = null;
      _items = <QuizItem>[];
      _at = 0;
      _grade = null;
    });
    _events = _cards.quiz(noteIds: widget.noteIds, run: run).listen((e) {
      if (!mounted) {
        return;
      }
      switch (e) {
        case QuizEvent_Status():
          break;
        case QuizEvent_Questions(:final items):
          setState(() {
            _items = items;
            _loading = false;
          });
        case QuizEvent_Failed(:final message):
          setState(() {
            _error = FlashcardsText.failure(message, context.l10n);
            _loading = false;
          });
        case QuizEvent_Stopped():
          setState(() => _loading = false);
      }
    });
  }

  void _check() {
    final item = _items[_at];
    final run = getIt<KansoAi>().newRun();
    _run = run;
    setState(() {
      _grading = true;
      _error = null;
    });
    _events = _cards
        .grade(
          question: item.question,
          noteId: item.noteId,
          answer: _answer.text,
          run: run,
        )
        .listen((e) {
          if (!mounted) {
            return;
          }
          switch (e) {
            case final GradeEvent_Graded graded:
              setState(() {
                _grade = graded;
                _grading = false;
              });
            case GradeEvent_Failed(:final message):
              setState(() {
                _error = FlashcardsText.failure(message, context.l10n);
                _grading = false;
              });
            case GradeEvent_Stopped():
              setState(() => _grading = false);
          }
        });
  }

  Future<void> _saveCard() async {
    final item = _items[_at];
    try {
      await _cards.addCards(
        noteId: item.noteId,
        cards: <CardPair>[
          CardPair(
            question: item.question,
            answer: _grade?.comment ?? item.source ?? '',
          ),
        ],
      );
      if (mounted) {
        setState(() => _saved = item.question);
      }
    } on Object catch (e) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(e).message);
      }
    }
  }

  void _next() {
    setState(() {
      _at += 1;
      _grade = null;
      _saved = null;
      _answer.clear();
    });
  }

  @override
  void dispose() {
    _run?.stop();
    unawaited(_events?.cancel());
    _answer.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final type = context.typography;
    final finished = !_loading && _items.isNotEmpty && _at >= _items.length;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        SheetHeader(
          title: context.l10n.flashcardsQuizMe,
          actionLabel: context.l10n.settingsClose,
        ),
        if (_loading)
          const Padding(
            padding: EdgeInsets.symmetric(vertical: KansoSpacing.lg),
            child: KansoProgressBar(),
          )
        else if (finished)
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.lg),
            child: Column(
              children: <Widget>[
                Text(context.l10n.flashcardsQuizDone, style: type.heading2),
                const SizedBox(height: KansoSpacing.lg),
                KansoButton(
                  label: context.l10n.flashcardsQuizAgain,
                  onPressed: _start,
                ),
              ],
            ),
          )
        else if (_items.isEmpty)
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: _error == null
                ? const SizedBox.shrink()
                : KansoInlineMessage(message: _error!),
          )
        else
          Flexible(child: _question(context)),
      ],
    );
  }

  Widget _question(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final item = _items[_at];
    final grade = _grade;
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md,
      ),
      children: <Widget>[
        Text(
          context.l10n.flashcardsQuizProgress(_at + 1, _items.length),
          style: type.meta.copyWith(color: colors.textSecondary),
        ),
        const SizedBox(height: KansoSpacing.xs),
        Text(item.question, style: type.heading2),
        const SizedBox(height: KansoSpacing.xxs),
        Text(
          item.noteTitle,
          style: type.meta.copyWith(color: colors.textSecondary),
        ),
        const SizedBox(height: KansoSpacing.md),
        KansoCard(
          padding: const EdgeInsets.all(KansoSpacing.sm),
          child: KansoTextField(
            controller: _answer,
            hint: context.l10n.flashcardsQuizHint,
            enabled: grade == null && !_grading,
            minLines: 3,
            maxLines: 8,
            keyboardType: TextInputType.multiline,
            textInputAction: TextInputAction.newline,
            autocorrect: true,
            onChanged: (_) => setState(() {}),
          ),
        ),
        const SizedBox(height: KansoSpacing.md),
        if (grade == null)
          KansoButton(
            label: context.l10n.flashcardsQuizCheck,
            onPressed: _grading || _answer.text.trim().isEmpty ? null : _check,
          )
        else ...<Widget>[
          KansoCard(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(grade.label, style: type.heading2),
                const SizedBox(height: KansoSpacing.xs),
                Text(grade.comment, style: type.body),
              ],
            ),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: _saved == item.question
                ? 'Saved'
                : context.l10n.flashcardsQuizSaveCard,
            variant: KansoButtonVariant.secondary,
            onPressed: _saved == item.question ? null : _saveCard,
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(label: context.l10n.flashcardsQuizNext, onPressed: _next),
        ],
        if (_grading)
          const Padding(
            padding: EdgeInsets.only(top: KansoSpacing.md),
            child: KansoProgressBar(),
          ),
        if (_error case final message?)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.sm),
            child: KansoInlineMessage(message: message),
          ),
      ],
    );
  }
}
