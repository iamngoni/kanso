// Kanso — triage_pager.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/triage_card_state.dart';
import '../../state/triage/triage_bloc.dart';
import 'triage_card_view.dart';

/// How far a card is dragged, in logical pixels, before letting go acts.
const double triageSwipeDistance = 96;

/// How fast a flick is, in logical pixels a second, to act without the
/// distance.
const double triageSwipeVelocity = 700;

/// The cards, one full screen at a time. A swipe right files the card
/// (when something is accepted on it and it has not changed; otherwise it
/// springs back), a swipe left skips it. Either way the next card slides
/// in, because the one acted on leaves the list. Inside a card the chips
/// scroll vertically; the horizontal drag belongs to the swipe.
class TriagePager extends StatefulWidget {
  const TriagePager({required this.cards, super.key});

  /// The cards still showing.
  final List<TriageCardState> cards;

  @override
  State<TriagePager> createState() => _TriagePagerState();
}

class _TriagePagerState extends State<TriagePager> {
  final PageController _controller = PageController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return PageView.builder(
      key: const ValueKey<String>('triage-pager'),
      controller: _controller,
      // Dragging sideways is the swipe, not a page turn: the list changes
      // under the pager when a card is filed or skipped.
      physics: const NeverScrollableScrollPhysics(),
      itemCount: widget.cards.length,
      findChildIndexCallback: (key) {
        if (key is ValueKey<String>) {
          final at = widget.cards.indexWhere((c) => c.noteId == key.value);
          return at < 0 ? null : at;
        }
        return null;
      },
      itemBuilder: (context, i) {
        final card = widget.cards[i];
        return _SwipeCard(
          key: ValueKey<String>(card.noteId),
          card: card,
          onFile: card.hasAccepted && !card.changed
              ? () => context.read<TriageBloc>().add(
                  TriageEvent.cardFiled(card.noteId),
                )
              : null,
          onSkip: () => context.read<TriageBloc>().add(
            TriageEvent.cardSkipped(card.noteId),
          ),
        );
      },
    );
  }
}

class _SwipeCard extends StatefulWidget {
  const _SwipeCard({
    required this.card,
    required this.onFile,
    required this.onSkip,
    super.key,
  });

  final TriageCardState card;

  /// Null while nothing is accepted or the card says the note changed.
  final VoidCallback? onFile;
  final VoidCallback onSkip;

  @override
  State<_SwipeCard> createState() => _SwipeCardState();
}

class _SwipeCardState extends State<_SwipeCard> {
  double _dx = 0;

  void _end(DragEndDetails details) {
    final velocity = details.primaryVelocity ?? 0;
    final right = _dx >= triageSwipeDistance || velocity >= triageSwipeVelocity;
    final left =
        _dx <= -triageSwipeDistance || velocity <= -triageSwipeVelocity;
    setState(() => _dx = 0);
    if (right) {
      widget.onFile?.call();
    } else if (left) {
      widget.onSkip();
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final filing = _dx > 0;
    final reach = (_dx.abs() / triageSwipeDistance).clamp(0.0, 1.0);
    final ready = !filing || widget.onFile != null;
    final label = filing ? l10n.triageFile : l10n.triageSkip;
    return GestureDetector(
      behavior: HitTestBehavior.translucent,
      onHorizontalDragUpdate: (d) =>
          setState(() => _dx = (_dx + d.delta.dx).clamp(-240.0, 240.0)),
      onHorizontalDragCancel: () => setState(() => _dx = 0),
      onHorizontalDragEnd: _end,
      child: Stack(
        children: <Widget>[
          if (_dx != 0)
            Positioned.fill(
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.lg,
                ),
                child: Align(
                  alignment: filing
                      ? Alignment.centerLeft
                      : Alignment.centerRight,
                  child: Opacity(
                    opacity: reach * (ready ? 1 : 0.4),
                    child: Text(
                      label,
                      style: context.typography.body.copyWith(
                        color: filing ? colors.accent : colors.textSecondary,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          Transform.translate(
            offset: Offset(_dx, 0),
            child: SingleChildScrollView(
              padding: const EdgeInsets.only(bottom: KansoSpacing.lg),
              child: TriageCardView(card: widget.card),
            ),
          ),
        ],
      ),
    );
  }
}
