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
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
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
    return ListView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      children: <Widget>[
        Text(l10n.historyVersionsTitle, style: context.typography.heading2),
        const SizedBox(height: KansoSpacing.xs),
        ListTile(
          title: Text(l10n.historyNow),
          subtitle: Text(history.currentTitle),
          selected: state.selectedRev == null,
        ),
        for (final version in history.versions)
          ListTile(
            title: Text(
              version.title.isEmpty ? l10n.replaceUntitled : version.title,
            ),
            subtitle: Text(
              '${MaterialLocalizations.of(context).formatMediumDate(
                DateTime.fromMillisecondsSinceEpoch(
                  version.savedAt,
                ).toLocal(),
              )} · ${version.originLabel}',
            ),
            selected: version.rev == state.selectedRev,
            onTap: () => context.read<NoteHistoryBloc>().add(
              NoteHistoryEvent.versionSelected(version.rev),
            ),
          ),
        if (selected != null) ...<Widget>[
          const Divider(),
          SegmentedButton<HistoryPane>(
            segments: <ButtonSegment<HistoryPane>>[
              ButtonSegment<HistoryPane>(
                value: HistoryPane.changes,
                label: Text(l10n.historyPaneChanges),
              ),
              ButtonSegment<HistoryPane>(
                value: HistoryPane.version,
                label: Text(l10n.historyPaneVersion),
              ),
            ],
            selected: <HistoryPane>{state.pane},
            onSelectionChanged: (value) => context.read<NoteHistoryBloc>().add(
              NoteHistoryEvent.paneChanged(value.first),
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
          if (state.problem != null)
            Text(switch (state.problem!) {
              HistoryProblem.changedMeanwhile => l10n.historyProblemChanged,
              HistoryProblem.versionGone => l10n.historyProblemGone,
              HistoryProblem.restoreFailed => l10n.historyProblemRestore,
              HistoryProblem.diffFailed => l10n.historyProblemDiff,
            }, style: TextStyle(color: context.colors.danger)),
          if (state.pane == HistoryPane.version)
            SelectableText('${selected.title}\n\n${selected.body}')
          else if (state.diffStatus == LoadStatus.loading)
            Semantics(
              label: l10n.historyDiffLoading,
              child: const LinearProgressIndicator(),
            )
          else if (state.hunks.isEmpty)
            Text(l10n.historyNoChanges)
          else
            for (final hunk in state.hunks)
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      for (final line in hunk.lines)
                        Text(
                          '${line.kind == DiffKind.added
                              ? '+ '
                              : line.kind == DiffKind.removed
                              ? '− '
                              : '  '}${line.text}',
                          style: TextStyle(
                            color: line.kind == DiffKind.added
                                ? context.colors.success
                                : line.kind == DiffKind.removed
                                ? context.colors.danger
                                : context.colors.text,
                          ),
                        ),
                      Align(
                        alignment: AlignmentDirectional.centerEnd,
                        child: TextButton.icon(
                          onPressed: () => context.read<NoteHistoryBloc>().add(
                            NoteHistoryEvent.sectionCopied(
                              state.hunks.indexOf(hunk),
                            ),
                          ),
                          icon: const Icon(Icons.copy),
                          label: Text(l10n.historyCopySection),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: state.restoring
                ? l10n.historyRestoring
                : l10n.historyRestore,
            icon: KansoIcons.undo,
            onPressed: state.restoring
                ? null
                : () => context.read<NoteHistoryBloc>().add(
                    const NoteHistoryEvent.restoreRequested(),
                  ),
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
    );
  }
}
