// Kanso — review_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_swipe_host.dart';
import '../../../injection.dart';
import '../../../tasks/views/widgets/task_row.dart';
import '../../utils/close_loop.dart';
import '../../utils/open_digest_note.dart';
import '../widgets/digest_loader.dart';
import '../widgets/loop_row.dart';

/// The weekly review: the seven days of a week (this one, or any before
/// it), what was written and finished, what is still open and the themes
/// that came up. Save as Note keeps a copy in the Journal.
class ReviewPage extends StatefulWidget {
  const ReviewPage({super.key});

  @override
  State<ReviewPage> createState() => _ReviewPageState();
}

class _ReviewPageState extends State<ReviewPage> {
  /// The Monday of the week shown; null for this week.
  String? from;

  /// The week shown as it was last read, to step from.
  ReviewView? shown;

  Future<void> step(int weeks) async {
    final current = shown;
    if (current == null) {
      return;
    }
    final start = DateTime.parse(current.from);
    final moved = DateTime.utc(start.year, start.month, start.day + 7 * weeks);
    final text =
        '${moved.year.toString().padLeft(4, '0')}-'
        '${moved.month.toString().padLeft(2, '0')}-'
        '${moved.day.toString().padLeft(2, '0')}';
    setState(() => from = text);
  }

  Future<void> save(ReviewView review) async {
    final l10n = context.l10n;
    final repo = getIt<DigestRepository>();
    var result = await repo.saveReview(from: review.from, replace: false);
    if (!mounted) {
      return;
    }
    final saved = result.fold<ReviewSaved?>((_) => null, (it) => it);
    if (saved != null && saved.exists) {
      final replace = await KansoConfirmSheet.show(
        context,
        title: l10n.digestReviewReplaceTitle,
        message: l10n.digestReviewReplaceMessage,
        confirmLabel: l10n.digestReviewReplace,
        destructive: false,
      );
      if (!replace || !mounted) {
        return;
      }
      result = await repo.saveReview(from: review.from, replace: true);
      if (!mounted) {
        return;
      }
    }
    result.fold(
      (_) => KansoFeedback.show(
        context,
        l10n.digestReviewError,
        tone: KansoFeedbackTone.error,
      ),
      (done) => KansoFeedback.show(
        context,
        l10n.digestReviewSaved,
        actionLabel: l10n.digestReviewOpenSaved,
        onAction: () => openDigestNote(context, done.noteId),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return DigestLoader<ReviewView>(
      title: l10n.digestReviewTitle,
      subtitle: (review) => review.rangeLabel,
      reloadKey: from,
      load: () async {
        final result = await getIt<DigestRepository>().review(from: from);
        result.fold((_) {}, (it) => shown = it);
        return result;
      },
      trailing: <Widget>[
        KansoIconButton(
          icon: KansoIcons.chevronLeft,
          semanticLabel: l10n.digestReviewPrevious,
          onPressed: () => unawaited(step(-1)),
        ),
        KansoIconButton(
          icon: KansoIcons.chevronRight,
          semanticLabel: l10n.digestReviewNext,
          onPressed: (shown?.isThisWeek ?? true)
              ? null
              : () => unawaited(step(1)),
        ),
      ],
      builder: (context, review, reload) {
        final empty =
            review.written.isEmpty &&
            review.finished.isEmpty &&
            review.stillOpen.isEmpty &&
            review.overdue.isEmpty &&
            review.next.isEmpty;
        Widget meta(String text) => Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xxs,
          ),
          child: Text(
            text,
            style: typography.body.copyWith(color: colors.text),
          ),
        );
        return KansoSwipeHost(
          child: ListView(
            padding: digestListPadding,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.md,
                  KansoSpacing.sm,
                  KansoSpacing.md,
                  0,
                ),
                child: Text(
                  '${l10n.digestReviewWeek(review.weekLabel)} · '
                  '${review.countsText}',
                  style: typography.meta.copyWith(color: colors.textMuted),
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: KansoButton(
                  label: l10n.digestReviewSave,
                  variant: KansoButtonVariant.secondary,
                  icon: KansoIcons.fileText,
                  onPressed: () => unawaited(save(review)),
                ),
              ),
              if (review.savedNoteId case final saved?)
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    0,
                    KansoSpacing.md,
                    KansoSpacing.sm,
                  ),
                  child: KansoPressable(
                    onTap: () => openDigestNote(context, saved),
                    child: Text(
                      l10n.digestReviewOpenSaved,
                      style: typography.body.copyWith(color: colors.accent),
                    ),
                  ),
                ),
              if (empty) meta(l10n.digestReviewNothing),
              if (review.written.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewWritten),
                for (final note in review.written)
                  KansoPressable(
                    onTap: () => openDigestNote(context, note.noteId),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                        vertical: KansoSpacing.xs,
                      ),
                      child: Row(
                        children: <Widget>[
                          Expanded(
                            child: Text(
                              note.title.trim().isEmpty
                                  ? l10n.noteUntitled
                                  : note.title,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.body.copyWith(
                                color: colors.text,
                              ),
                            ),
                          ),
                          Text(
                            note.notebookName,
                            style: typography.meta.copyWith(
                              color: colors.textMuted,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
              ],
              if (review.finished.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewFinished),
                for (final done in review.finished)
                  KansoPressable(
                    onTap: () => openDigestNote(context, done.noteId),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                        vertical: KansoSpacing.xs,
                      ),
                      child: Text(
                        done.text,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(
                          color: colors.textMuted,
                          decoration: TextDecoration.lineThrough,
                          decorationColor: colors.textMuted,
                        ),
                      ),
                    ),
                  ),
              ],
              if (review.stillOpen.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewStillOpen),
                for (final item in review.stillOpen)
                  LoopRow(
                    key: ValueKey<String>('${item.noteId}/${item.key}'),
                    item: item,
                    onAction: (action) => closeLoop(context, item, action),
                    onOpen: () =>
                        openDigestNote(context, item.noteId, line: item.line),
                  ),
              ],
              if (review.overdue.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewOverdue),
                for (final task in review.overdue) _task(context, task),
              ],
              if (review.themes.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewThemes),
                for (final theme in review.themes)
                  meta(
                    l10n.digestReviewThemeNotes(
                      theme.isTag ? '#${theme.term}' : theme.term,
                      theme.notes,
                    ),
                  ),
              ],
              if (review.next.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestReviewNextWeek),
                for (final task in review.next) _task(context, task),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _task(BuildContext context, TaskItem task) => TaskRow(
    task: task,
    onTick: () => unawaited(
      getIt<LibraryRepository>().setTask(
        noteId: task.noteId,
        line: task.line,
        text: task.sourceText,
        checked: !task.checked,
      ),
    ),
    onOpen: () => openDigestNote(context, task.noteId, line: task.line),
  );
}
