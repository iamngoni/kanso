// Kanso — ask_turn_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/ask_outcome.dart';
import '../../models/data/ask_phase.dart';
import '../../models/data/ask_turn.dart';
import 'answer_view.dart';
import 'used_notes_list.dart';

/// One question and its answer: the question, what is being done (searching,
/// thinking), the answer as it is written, how it ended, and the notes it
/// used.
class AskTurnView extends StatelessWidget {
  const AskTurnView({
    required this.turn,
    required this.onOpenNote,
    required this.onOpenLink,
    this.phase = AskPhase.idle,
    this.onRetry,
    this.usedExpanded = false,
    super.key,
  });

  final AskTurn turn;

  /// What the conversation is doing, if this is the question being answered.
  final AskPhase phase;
  final ValueChanged<String> onOpenNote;
  final ValueChanged<String> onOpenLink;

  /// Asks the question again; offered when it failed and this is the last.
  final VoidCallback? onRetry;

  /// Whether the list of notes used starts open.
  final bool usedExpanded;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final quiet = typography.secondary;
    final pending = turn.outcome == AskOutcome.pending;
    final status = switch ((pending, phase)) {
      (true, AskPhase.searching) => l10n.askSearching,
      (true, AskPhase.thinking) => l10n.askThinking,
      (true, AskPhase.answering) when turn.answer.isEmpty => l10n.askThinking,
      _ => null,
    };
    final notice = switch (turn.outcome) {
      AskOutcome.nothingMatches => l10n.askNothingMatches,
      AskOutcome.cutOff => l10n.askCutOff,
      AskOutcome.stopped => l10n.askStopped,
      _ => null,
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Align(
          alignment: AlignmentDirectional.centerEnd,
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            decoration: BoxDecoration(
              color: colors.fill,
              borderRadius: KansoRadii.largeBorder,
            ),
            child: Text(turn.question, style: typography.body),
          ),
        ),
        const SizedBox(height: KansoSpacing.sm),
        if (status != null)
          Semantics(
            liveRegion: true,
            child: Text(status, style: quiet),
          ),
        if (turn.answer.isNotEmpty)
          AnswerView(
            markdown: turn.answer,
            onOpenNote: onOpenNote,
            onOpenLink: onOpenLink,
            usedNoteIds: turn.used.map((note) => note.id).toSet(),
          ),
        if (notice != null) ...<Widget>[
          if (turn.answer.isNotEmpty) const SizedBox(height: KansoSpacing.xs),
          Text(notice, style: quiet),
        ],
        if (turn.outcome == AskOutcome.failed)
          Center(
            child: KansoMessageCard(
              icon: KansoIcons.circleX,
              title: l10n.askFailedTitle,
              message: turn.failure ?? l10n.askFailedMessage,
              actions: <Widget>[
                if (onRetry != null)
                  KansoButton(label: l10n.retry, onPressed: onRetry),
              ],
            ),
          ),
        if (turn.used.isNotEmpty)
          UsedNotesList(
            notes: turn.used,
            onOpen: onOpenNote,
            initiallyExpanded: usedExpanded,
          ),
      ],
    );
  }
}
