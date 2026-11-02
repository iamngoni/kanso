// Kanso — quiz_page.dart
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
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../utils/flashcards_text.dart';

/// Quiz Me, at `/quiz`: five questions from [noteIds], answered in words and
/// graded by the model against the note the question came from. Nothing is
/// stored and the review schedule is not touched; **Save as Card** turns a
/// question into a card in its note.
class QuizPage extends StatefulWidget {
  const QuizPage({required this.noteIds, super.key});

  final List<String> noteIds;

  @override
  State<QuizPage> createState() => _QuizPageState();
}

class _QuizPageState extends State<QuizPage> {
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
            _error = message;
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
                _error = message;
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
    final colors = context.colors;
    final type = context.typography;
    final finished = !_loading && _items.isNotEmpty && _at >= _items.length;
    return KansoPage(
      backgroundColor: colors.app,
      body: ColoredBox(
        color: colors.app,
        child: Column(
          children: <Widget>[
            KansoTopBar(
              backgroundColor: colors.app,
              bordered: false,
              leading: const ShellBackButton(),
              title: FlashcardsText.quizMe,
            ),
            Expanded(
              child: _loading
                  ? const Center(child: CircularProgressIndicator())
                  : finished
                  ? Padding(
                      padding: const EdgeInsets.all(KansoSpacing.lg),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: <Widget>[
                          Text(FlashcardsText.quizDone, style: type.heading2),
                          const SizedBox(height: KansoSpacing.lg),
                          KansoButton(
                            label: FlashcardsText.quizAgain,
                            onPressed: _start,
                          ),
                        ],
                      ),
                    )
                  : _items.isEmpty
                  ? Padding(
                      padding: const EdgeInsets.all(KansoSpacing.lg),
                      child: Text(_error ?? '', style: type.body),
                    )
                  : _question(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _question(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final item = _items[_at];
    final grade = _grade;
    return ListView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      children: <Widget>[
        Text(
          FlashcardsText.quizProgress(_at + 1, _items.length),
          style: type.meta.copyWith(color: colors.textSecondary),
        ),
        const SizedBox(height: KansoSpacing.sm),
        Text(item.question, style: type.heading1),
        Text(
          item.noteTitle,
          style: type.meta.copyWith(color: colors.textSecondary),
        ),
        const SizedBox(height: KansoSpacing.md),
        TextField(
          controller: _answer,
          enabled: grade == null,
          minLines: 3,
          maxLines: 8,
          decoration: const InputDecoration(hintText: FlashcardsText.quizHint),
        ),
        const SizedBox(height: KansoSpacing.md),
        if (grade == null)
          KansoButton(
            label: FlashcardsText.quizCheck,
            onPressed: _grading || _answer.text.trim().isEmpty ? null : _check,
          )
        else ...<Widget>[
          Text(grade.label, style: type.heading2),
          const SizedBox(height: KansoSpacing.xs),
          Text(grade.comment, style: type.body),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: _saved == item.question
                ? 'Saved'
                : FlashcardsText.quizSaveCard,
            variant: KansoButtonVariant.secondary,
            onPressed: _saved == item.question ? null : _saveCard,
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(label: FlashcardsText.quizNext, onPressed: _next),
        ],
        if (_grading)
          const Padding(
            padding: EdgeInsets.only(top: KansoSpacing.md),
            child: Center(child: CircularProgressIndicator()),
          ),
        if (_error case final message?)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.sm),
            child: Text(
              message,
              style: type.meta.copyWith(color: colors.danger),
            ),
          ),
      ],
    );
  }
}
