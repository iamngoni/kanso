// Kanso — ask_turn_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/ask_receipt.dart';
import '../../../core/utils/ai_trouble_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/ask_outcome.dart';
import '../../models/data/ask_phase.dart';
import '../../models/data/ask_turn.dart';
import 'answer_view.dart';
import 'sources_list.dart';
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
    this.onOpenReceipt,
    this.onJumpToReceipt,
    this.onSearchAll,
    this.onDropTime,
    this.onSave,
    this.saveFailure,
    this.usedExpanded = false,
    super.key,
  });

  /// Shows the lines a receipt points at.
  final ValueChanged<AskReceipt>? onOpenReceipt;

  /// Opens the note at the lines a receipt points at.
  final ValueChanged<AskReceipt>? onJumpToReceipt;

  /// Asks the question again over all notes; offered when a narrow scope
  /// matched nothing and this is the last.
  final VoidCallback? onSearchAll;

  /// Asks again without the time range; offered on the last turn.
  final VoidCallback? onDropTime;

  /// Saves the answer as a note; offered on the last answer.
  final VoidCallback? onSave;
  final String? saveFailure;

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
    AskReceipt? receipt(int n) {
      for (final candidate in turn.receipts) {
        if (candidate.n == n) {
          return candidate;
        }
      }
      return null;
    }

    final answered = turn.outcome == AskOutcome.answered;
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
        if (turn.notice.isNotEmpty) ...<Widget>[
          Text(turn.notice, style: quiet),
          const SizedBox(height: KansoSpacing.xs),
        ],
        if (turn.time != null) ...<Widget>[
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: Chip(
              label: Text(turn.time!.label),
              deleteIcon: const Icon(Icons.close, size: KansoSizes.iconSmall),
              deleteButtonTooltipMessage: l10n.askDropTime,
              onDeleted: onDropTime,
            ),
          ),
          if (turn.time!.relaxedLine.isNotEmpty)
            Text(turn.time!.relaxedLine, style: quiet),
        ],
        if (turn.banner.isNotEmpty && answered)
          Padding(
            padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
            child: Text(
              turn.banner,
              style: quiet.copyWith(color: colors.warning),
            ),
          ),
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
            onOpenReceipt: (n) {
              final found = receipt(n);
              if (found != null) {
                onOpenReceipt?.call(found);
              }
            },
            onJumpToReceipt: (n) {
              final found = receipt(n);
              if (found != null) {
                onJumpToReceipt?.call(found);
              }
            },
          ),
        if (answered && turn.neverWrote) ...<Widget>[
          const SizedBox(height: KansoSpacing.xs),
          Text(l10n.askNeverWrote, style: typography.listTitle),
          if (turn.closest.isNotEmpty) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            Text(l10n.askClosestNotes, style: quiet),
            for (final near in turn.closest)
              TextButton(
                onPressed: () => onOpenNote(near.noteId),
                child: Align(
                  alignment: AlignmentDirectional.centerStart,
                  child: Text(near.title),
                ),
              ),
          ],
        ],
        if (answered && turn.noSource > 0)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.xs),
            child: Text(l10n.askNoSource(turn.noSource), style: quiet),
          ),
        if (answered && turn.receipts.isNotEmpty)
          SourcesList(
            receipts: turn.receipts,
            onOpen: (found) => onOpenReceipt?.call(found),
          ),
        if (answered && turn.alsoRead.isNotEmpty) ...<Widget>[
          const SizedBox(height: KansoSpacing.xs),
          Text(l10n.askAlsoRead, style: quiet),
          for (final note in turn.alsoRead)
            TextButton(
              onPressed: () => onOpenNote(note.id),
              child: Align(
                alignment: AlignmentDirectional.centerStart,
                child: Text(note.title),
              ),
            ),
        ],
        if (notice != null) ...<Widget>[
          if (turn.answer.isNotEmpty) const SizedBox(height: KansoSpacing.xs),
          Text(notice, style: quiet),
        ],
        if (turn.outcome == AskOutcome.nothingMatches &&
            turn.narrow &&
            onSearchAll != null)
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: KansoButton(
              label: l10n.askSearchAll,
              expanded: false,
              onPressed: onSearchAll,
            ),
          ),
        if (answered && onSave != null && !turn.neverWrote)
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: turn.savedNoteId != null
                ? TextButton(
                    onPressed: () => onOpenNote(turn.savedNoteId!),
                    child: Text(l10n.askSavedAsNote),
                  )
                : KansoButton(
                    label: l10n.askSaveAsNote,
                    expanded: false,
                    onPressed: onSave,
                  ),
          ),
        if (saveFailure != null)
          Text(aiTroubleText(saveFailure!, l10n), style: quiet),
        if (turn.outcome == AskOutcome.failed)
          Center(
            child: KansoMessageCard(
              icon: KansoIcons.circleX,
              title: l10n.askFailedTitle,
              message: turn.failure == null
                  ? l10n.askFailedMessage
                  : aiTroubleText(turn.failure!, l10n),
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
