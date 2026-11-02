// Kanso — brief_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_swipe_host.dart';
import '../../../injection.dart';
import '../../../tasks/views/widgets/task_row.dart';
import '../../utils/close_loop.dart';
import '../../utils/open_digest_note.dart';
import '../widgets/digest_loader.dart';
import '../widgets/loop_row.dart';

/// The morning brief: what is due, the open loops, the timed tasks still to
/// come today and past notes that matter today. It is worked out when the
/// page opens and again when the library changes.
class BriefPage extends StatelessWidget {
  const BriefPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return DigestLoader<BriefView>(
      title: l10n.digestBriefTitle,
      subtitle: (brief) => brief.dateLabel,
      load: getIt<DigestRepository>().brief,
      builder: (context, brief, reload) {
        if (brief.isEmpty && brief.relevant.isEmpty) {
          return KansoEmptyState(
            title: l10n.digestBriefEmptyTitle,
            message: l10n.digestBriefEmptyMessage,
          );
        }
        final typography = context.typography;
        final colors = context.colors;
        return KansoSwipeHost(
          child: ListView(
            padding: digestListPadding,
            children: <Widget>[
              if (brief.due.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestBriefDue),
                for (final task in brief.due)
                  TaskRow(
                    task: task,
                    onTick: () => unawaited(_tick(task)),
                    onOpen: () =>
                        openDigestNote(context, task.noteId, line: task.line),
                  ),
              ],
              if (brief.laterToday.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestBriefLater),
                for (final later in brief.laterToday)
                  KansoPressable(
                    onTap: () =>
                        openDigestNote(context, later.noteId, line: later.line),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                        vertical: KansoSpacing.sm,
                      ),
                      child: Row(
                        children: <Widget>[
                          Text(
                            later.timeLabel,
                            style: typography.meta.copyWith(
                              color: colors.accent,
                            ),
                          ),
                          const SizedBox(width: KansoSpacing.sm),
                          Expanded(
                            child: Text(
                              later.text,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                              style: typography.body.copyWith(
                                color: colors.text,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
              ],
              if (brief.loops.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestBriefLoops),
                for (final item in brief.loops)
                  LoopRow(
                    key: ValueKey<String>('${item.noteId}/${item.key}'),
                    item: item,
                    onAction: (action) => closeLoop(context, item, action),
                    onOpen: () =>
                        openDigestNote(context, item.noteId, line: item.line),
                  ),
                if (brief.loopsTotal > brief.loops.length)
                  Padding(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    child: KansoButton(
                      label: l10n.digestBriefMoreLoops(brief.loopsTotal),
                      variant: KansoButtonVariant.secondary,
                      onPressed: () => context.push(KansoRoutes.loops),
                    ),
                  ),
              ],
              if (brief.relevant.isNotEmpty) ...<Widget>[
                KansoSectionHeader(label: l10n.digestBriefRelevant),
                for (final note in brief.relevant)
                  KansoPressable(
                    onTap: () => openDigestNote(context, note.noteId),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                        vertical: KansoSpacing.sm,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: <Widget>[
                          Text(
                            note.title.trim().isEmpty
                                ? l10n.noteUntitled
                                : note.title,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: typography.body.copyWith(
                              color: colors.text,
                            ),
                          ),
                          Text(
                            note.why,
                            style: typography.meta.copyWith(
                              color: colors.accent,
                            ),
                          ),
                          if (note.excerpt.trim().isNotEmpty)
                            Text(
                              note.excerpt,
                              maxLines: 2,
                              overflow: TextOverflow.ellipsis,
                              style: typography.meta.copyWith(
                                color: colors.textMuted,
                              ),
                            ),
                        ],
                      ),
                    ),
                  ),
              ],
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: Text(
                  l10n.digestBriefYesterday(
                    brief.wroteYesterday,
                    brief.finishedYesterday,
                  ),
                  style: typography.meta.copyWith(color: colors.textMuted),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Future<void> _tick(TaskItem task) async {
    await getIt<LibraryRepository>().setTask(
      noteId: task.noteId,
      line: task.line,
      text: task.sourceText,
      checked: !task.checked,
    );
  }
}
