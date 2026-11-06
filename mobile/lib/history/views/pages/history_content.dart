// Kanso — history_content.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/list_labels.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_card.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../models/data/history_pane.dart';
import '../../models/data/history_problem.dart';
import '../../state/note_history/note_history_bloc.dart';

/// The version list and the diff for the chosen version.
class HistoryContent extends StatelessWidget {
  const HistoryContent({required this.state, super.key});

  final NoteHistoryState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final history = state.history;
    if (history == null) return const SizedBox.shrink();
    final selected = history.versions
        .where((version) => version.rev == state.selectedRev)
        .firstOrNull;
    final bloc = context.read<NoteHistoryBloc>();
    return KansoListScrollbar(
      child: ListView(
        padding: const EdgeInsets.fromLTRB(
          KansoSpacing.md,
          KansoSpacing.md,
          KansoSpacing.md,
          KansoSpacing.xxl,
        ),
        children: <Widget>[
          KansoSectionHeader(
            label: l10n.historyVersionsTitle,
            horizontalPadding: 0,
          ),
          KansoGroupedSection(
            children: <Widget>[
              _VersionRow(
                title: l10n.historyNow,
                detail: history.currentTitle,
                selected: state.selectedRev == null,
                onTap: null,
              ),
              for (final version in history.versions)
                _VersionRow(
                  title: version.title.isEmpty
                      ? l10n.replaceUntitled
                      : version.title,
                  detail:
                      '${dateLabel(l10n, version.savedAt)} · '
                      '${version.originLabel}',
                  selected: version.rev == state.selectedRev,
                  onTap: () => bloc.add(
                    NoteHistoryEvent.versionSelected(version.rev),
                  ),
                ),
            ],
          ),
          if (selected != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.lg),
            KansoSegmented<HistoryPane>(
              selected: state.pane,
              semanticLabel: l10n.historyVersionsTitle,
              segments: <KansoSegment<HistoryPane>>[
                KansoSegment<HistoryPane>(
                  value: HistoryPane.changes,
                  label: l10n.historyPaneChanges,
                ),
                KansoSegment<HistoryPane>(
                  value: HistoryPane.version,
                  label: l10n.historyPaneVersion,
                ),
              ],
              onChanged: (value) =>
                  bloc.add(NoteHistoryEvent.paneChanged(value)),
            ),
            const SizedBox(height: KansoSpacing.md),
            if (state.problem != null)
              Padding(
                padding: const EdgeInsets.only(bottom: KansoSpacing.md),
                child: KansoInlineMessage(
                  message: switch (state.problem!) {
                    HistoryProblem.changedMeanwhile =>
                      l10n.historyProblemChanged,
                    HistoryProblem.versionGone => l10n.historyProblemGone,
                    HistoryProblem.restoreFailed => l10n.historyProblemRestore,
                    HistoryProblem.diffFailed => l10n.historyProblemDiff,
                  },
                ),
              ),
            if (state.pane == HistoryPane.version)
              KansoCard(
                child: SelectableText(
                  '${selected.title}\n\n${selected.body}',
                  style: context.typography.body,
                ),
              )
            else if (state.diffStatus == LoadStatus.loading)
              Semantics(
                label: l10n.historyDiffLoading,
                child: const KansoProgressBar(),
              )
            else if (state.hunks.isEmpty)
              Text(l10n.historyNoChanges, style: context.typography.secondary)
            else
              for (var i = 0; i < state.hunks.length; i++) ...<Widget>[
                if (i > 0) const SizedBox(height: KansoSpacing.sm),
                _DiffHunkCard(
                  hunk: state.hunks[i],
                  onCopy: () => bloc.add(NoteHistoryEvent.sectionCopied(i)),
                ),
              ],
            const SizedBox(height: KansoSpacing.lg),
            KansoButton(
              label: state.restoring
                  ? l10n.historyRestoring
                  : l10n.historyRestore,
              icon: KansoIcons.undo,
              onPressed: state.restoring
                  ? null
                  : () => bloc.add(const NoteHistoryEvent.restoreRequested()),
            ),
          ],
          if (history.versions.isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xl),
              child: KansoMessageCard(
                icon: KansoIcons.clock,
                title: l10n.historyEmptyTitle,
                message: l10n.historyEmptyMessage,
                actions: const <Widget>[],
              ),
            ),
        ],
      ),
    );
  }
}

/// One version in the list: its title, and the day and where it came from.
///
/// [selected] puts a quiet wash behind it and an accent rule down its
/// leading edge, the same mark the list uses for the note that is open.
class _VersionRow extends StatelessWidget {
  const _VersionRow({
    required this.title,
    required this.detail,
    required this.selected,
    required this.onTap,
  });

  final String title;
  final String detail;
  final bool selected;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      selected: selected,
      child: KansoPressable(
        onTap: onTap,
        child: Stack(
          children: <Widget>[
            if (selected)
              Positioned.fill(
                child: ColoredBox(
                  color: colors.selected.withValues(alpha: 0.6),
                ),
              ),
            Padding(
              padding: const EdgeInsetsDirectional.only(
                start: KansoSpacing.md,
                end: KansoSpacing.md,
                top: KansoSpacing.sm,
                bottom: KansoSpacing.sm,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(
                    title,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.listTitle,
                  ),
                  const SizedBox(height: 2),
                  Text(
                    detail,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.meta,
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// One block of a diff: its lines in their colours, and a way to take just
/// this block.
class _DiffHunkCard extends StatelessWidget {
  const _DiffHunkCard({required this.hunk, required this.onCopy});

  final DiffHunk hunk;
  final VoidCallback onCopy;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    return KansoCard(
      padding: const EdgeInsets.all(KansoSpacing.sm),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          for (final line in hunk.lines)
            Text(
              '${switch (line.kind) {
                DiffKind.added => '+ ',
                DiffKind.removed => '− ',
                DiffKind.same => '  ',
              }}${line.text}',
              style: context.typography.mono.copyWith(
                color: switch (line.kind) {
                  DiffKind.added => colors.success,
                  DiffKind.removed => colors.danger,
                  DiffKind.same => colors.text,
                },
              ),
            ),
          const SizedBox(height: KansoSpacing.xs),
          Align(
            alignment: AlignmentDirectional.centerEnd,
            child: KansoButton(
              label: l10n.historyCopySection,
              variant: KansoButtonVariant.quiet,
              icon: KansoIcons.copy,
              compact: true,
              expanded: false,
              onPressed: onCopy,
            ),
          ),
        ],
      ),
    );
  }
}
