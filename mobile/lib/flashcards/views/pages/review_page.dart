// Kanso — review_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/cards.dart';
import '../../../core/bridge/api/cards_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../utils/flashcards_text.dart';
import '../widgets/review_stats_view.dart';

/// Review, at `/review`: the cards that are due, one at a time. A tap shows
/// the answer, then four buttons (with their next intervals) record how it
/// went; swiping left answers Again and right answers Good.
class FlashcardReviewPage extends StatefulWidget {
  const FlashcardReviewPage({this.scope, super.key});

  /// Which cards to review; all of them by default.
  final CardScopeArg? scope;

  @override
  State<FlashcardReviewPage> createState() => _ReviewPageState();
}

class _ReviewPageState extends State<FlashcardReviewPage> {
  late final KansoCards _cards = getIt<KansoLibrary>().cards();
  final List<ReviewCard> _queue = <ReviewCard>[];
  int _answered = 0;
  int _total = 0;
  bool _revealed = false;
  bool _loading = true;
  bool _stats = false;
  String? _error;

  CardScopeArg get _scope => widget.scope ?? FlashcardsText.everything;
  int get _offset => DateTime.now().timeZoneOffset.inMinutes;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final due = await _cards.due(
        scope: _scope,
        limit: 500,
        utcOffsetMinutes: _offset,
      );
      if (!mounted) {
        return;
      }
      setState(() {
        _queue
          ..clear()
          ..addAll(due);
        _answered = 0;
        _total = due.length;
        _revealed = false;
        _loading = false;
      });
    } on Object catch (error) {
      if (mounted) {
        setState(() {
          _error = applicationErrorOf(error).message;
          _loading = false;
        });
      }
    }
  }

  Future<void> _answer(int grade) async {
    if (_queue.isEmpty) {
      return;
    }
    final card = _queue.first;
    unawaited(HapticFeedback.lightImpact());
    try {
      await _cards.answer(
        noteId: card.noteId,
        cardKey: card.cardKey,
        grade: grade,
        utcOffsetMinutes: _offset,
      );
    } on Object catch (error) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(error).message);
      }
      return;
    }
    if (!mounted) {
      return;
    }
    setState(() {
      _queue.removeAt(0);
      _answered += 1;
      _revealed = false;
      _error = null;
      if (grade == 1) {
        // A card you missed comes back in this session.
        _queue.add(card);
        _total += 1;
      }
    });
  }

  Future<void> _suspend() async {
    if (_queue.isEmpty) {
      return;
    }
    final card = _queue.first;
    try {
      await _cards.suspend(
        noteId: card.noteId,
        cardKey: card.cardKey,
        suspended: true,
      );
    } on Object catch (error) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(error).message);
      }
      return;
    }
    if (mounted) {
      setState(() {
        _queue.removeAt(0);
        _revealed = false;
        _total -= 1;
      });
    }
  }

  void _openNote() {
    if (_queue.isEmpty) {
      return;
    }
    context.push(
      '${KansoRoutes.noteById}/${Uri.encodeComponent(_queue.first.noteId)}',
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
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
              title: _stats ? FlashcardsText.stats : FlashcardsText.reviewTitle,
              trailing: <Widget>[
                KansoIconButton(
                  icon: KansoIcons.chartLine,
                  semanticLabel: FlashcardsText.stats,
                  selected: _stats,
                  onPressed: () => setState(() => _stats = !_stats),
                ),
                KansoIconButton(
                  icon: KansoIcons.settings,
                  semanticLabel: FlashcardsText.settings,
                  onPressed: () => context.push(KansoRoutes.flashcardsSettings),
                ),
              ],
            ),
            Expanded(
              child: _stats ? ReviewStatsView(scope: _scope) : _body(context),
            ),
          ],
        ),
      ),
    );
  }

  Widget _body(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_queue.isEmpty) {
      return _Finished(
        answered: _answered,
        error: _error,
        onMore: _load,
      );
    }
    final card = _queue.first;
    final colors = context.colors;
    final type = context.typography;
    final fresh = _queue.where((c) => c.isNew).length;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: _revealed ? null : () => setState(() => _revealed = true),
      onHorizontalDragEnd: !_revealed
          ? null
          : (details) {
              final v = details.primaryVelocity ?? 0;
              if (v < -300) {
                _answer(1);
              } else if (v > 300) {
                _answer(3);
              }
            },
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Text(
              FlashcardsText.progress(_answered + 1, _total, fresh),
              style: type.meta.copyWith(color: colors.textSecondary),
            ),
            Expanded(
              child: SingleChildScrollView(
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    vertical: KansoSpacing.lg,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(card.question, style: type.heading1),
                      if (_revealed) ...<Widget>[
                        const SizedBox(height: KansoSpacing.lg),
                        Divider(color: colors.divider),
                        const SizedBox(height: KansoSpacing.md),
                        Text(card.answer, style: type.body),
                        const SizedBox(height: KansoSpacing.md),
                        TextButton(
                          onPressed: _openNote,
                          child: Text(
                            card.noteTitle.isEmpty
                                ? FlashcardsText.openNote
                                : card.noteTitle,
                            style: type.secondary.copyWith(color: colors.link),
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
              ),
            ),
            if (_error case final message?)
              Padding(
                padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
                child: Text(
                  message,
                  style: type.meta.copyWith(color: colors.danger),
                ),
              ),
            if (!_revealed)
              KansoButton(
                label: FlashcardsText.showAnswer,
                onPressed: () => setState(() => _revealed = true),
              )
            else ...<Widget>[
              Row(
                children: <Widget>[
                  for (var i = 0; i < 4; i++) ...<Widget>[
                    if (i > 0) const SizedBox(width: KansoSpacing.xs),
                    Expanded(
                      child: _GradeButton(
                        name: FlashcardsText.grades[i],
                        interval: card.intervals.length > i
                            ? card.intervals[i]
                            : '',
                        onPressed: () => _answer(i + 1),
                      ),
                    ),
                  ],
                ],
              ),
              const SizedBox(height: KansoSpacing.xs),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: <Widget>[
                  Text(
                    FlashcardsText.swipeHint,
                    style: type.meta.copyWith(color: colors.textMuted),
                  ),
                  TextButton(
                    onPressed: _suspend,
                    child: Text(
                      FlashcardsText.suspend,
                      style: type.meta.copyWith(color: colors.textSecondary),
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _GradeButton extends StatelessWidget {
  const _GradeButton({
    required this.name,
    required this.interval,
    required this.onPressed,
  });

  final String name;
  final String interval;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    return Material(
      color: colors.elevated,
      borderRadius: BorderRadius.circular(12),
      child: InkWell(
        borderRadius: BorderRadius.circular(12),
        onTap: onPressed,
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: KansoSpacing.sm),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text(name, style: type.label),
              const SizedBox(height: 2),
              Text(
                interval,
                style: type.meta.copyWith(color: colors.textSecondary),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _Finished extends StatelessWidget {
  const _Finished({
    required this.answered,
    required this.error,
    required this.onMore,
  });

  final int answered;
  final String? error;
  final VoidCallback onMore;

  @override
  Widget build(BuildContext context) {
    final none = answered == 0 && error == null;
    return Padding(
      padding: const EdgeInsets.all(KansoSpacing.lg),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: <Widget>[
          KansoEmptyState(
            title: error != null
                ? error!
                : none
                ? FlashcardsText.noCardsTitle
                : FlashcardsText.doneForToday,
            message: none
                ? FlashcardsText.noCardsBody
                : FlashcardsText.doneBody,
          ),
          const SizedBox(height: KansoSpacing.lg),
          KansoButton(
            label: FlashcardsText.reviewMore,
            variant: KansoButtonVariant.secondary,
            expanded: false,
            onPressed: onMore,
          ),
        ],
      ),
    );
  }
}
