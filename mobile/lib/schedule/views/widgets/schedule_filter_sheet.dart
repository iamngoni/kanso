// Kanso — schedule_filter_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../state/schedule/schedule_bloc.dart';

/// What the Schedule shows, as the desktop's filter menus have it: which
/// done tasks show, which date properties place a note, and which notebooks. What is chosen applies at once; the sheet stays until it is done
/// with.
class ScheduleFilterSheet extends StatelessWidget {
  const ScheduleFilterSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<ScheduleBloc>();
    return BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
      builder: (context, index) {
        return BlocBuilder<ScheduleBloc, ScheduleState>(
          builder: (context, state) {
            return ListView(
              shrinkWrap: true,
              padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
              children: <Widget>[
                Padding(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.md,
                    top: KansoSpacing.xs,
                  ),
                  child: Row(
                    children: <Widget>[
                      Expanded(
                        child: Text(
                          l10n.scheduleFilterTitle,
                          style: context.typography.barTitle,
                        ),
                      ),
                      KansoPressable(
                        onTap: () => Navigator.of(context).maybePop(),
                        child: Padding(
                          padding: const EdgeInsets.all(KansoSpacing.sm),
                          child: Text(
                            l10n.noteListDone,
                            style: context.typography.body.copyWith(
                              color: colors.accent,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                KansoSheetGroup(
                  children: <Widget>[
                    KansoCheckRow(
                      label: l10n.scheduleShowCompleted,
                      checked: state.showDone,
                      onTap: () => bloc.add(
                        ScheduleEvent.showDoneChanged(
                          showDone: !state.showDone,
                        ),
                      ),
                    ),
                  ],
                ),
                KansoSheetGroup(
                  title: l10n.scheduleDatesHeader,
                  children: <Widget>[
                    KansoCheckRow(
                      label: l10n.scheduleAllDates,
                      checked: state.keys.isEmpty,
                      onTap: () => bloc.add(const ScheduleEvent.keysCleared()),
                    ),
                    if (state.dateKeys.isEmpty)
                      KansoCheckRow(
                        label: l10n.scheduleNoDateProperties,
                        checked: false,
                      ),
                    for (final key in state.dateKeys)
                      KansoCheckRow(
                        label: key,
                        checked: state.keys.contains(key),
                        onTap: () => bloc.add(ScheduleEvent.keyToggled(key)),
                      ),
                  ],
                ),
                KansoSheetGroup(
                  title: l10n.scheduleNotebooksHeader,
                  children: <Widget>[
                    KansoCheckRow(
                      label: l10n.scheduleAllNotebooks,
                      checked: state.books.isEmpty,
                      onTap: () =>
                          bloc.add(const ScheduleEvent.notebooksCleared()),
                    ),
                    for (final notebook in index.notebooks)
                      KansoCheckRow(
                        label: notebook.role == NotebookRole.templates
                            ? l10n.libraryTemplates
                            : notebook.name,
                        checked: state.books.contains(notebook.id),
                        indent: notebook.depth * KansoSpacing.md,
                        onTap: () => bloc.add(
                          ScheduleEvent.notebookToggled(notebook.id),
                        ),
                      ),
                  ],
                ),
              ],
            );
          },
        );
      },
    );
  }
}
