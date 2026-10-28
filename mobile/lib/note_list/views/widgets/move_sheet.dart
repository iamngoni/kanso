// Kanso — move_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../state/note_list/note_list_bloc.dart';

/// The notebooks, in the order of the library, to move the notes [ids] to.
/// A notebook that holds all of them already is checked; a locked notebook
/// is dimmed, and cannot be chosen here.
///
/// Choosing one moves the notes in one write and closes the sheet. Notes in
/// the trash are restored by moving them out.
class MoveSheet extends StatelessWidget {
  const MoveSheet({required this.ids, super.key});

  final List<String> ids;

  /// Shows the sheet for [ids] over the whole app. [bloc] is the list the
  /// notes are in.
  static Future<void> show(
    BuildContext context, {
    required NoteListBloc bloc,
    required List<String> ids,
  }) {
    final index = context.read<LibraryIndexBloc>();
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<NoteListBloc>.value(value: bloc),
          BlocProvider<LibraryIndexBloc>.value(value: index),
        ],
        child: MoveSheet(ids: ids),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<NoteListBloc>();
    final notebooks = context.select<LibraryIndexBloc, List<NotebookRow>>(
      (index) => index.state.notebooks,
    );
    final holding = <String>{
      for (final row in bloc.state.rows)
        if (ids.contains(row.id)) row.notebookId,
    };
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
                  l10n.moveSheetTitle(ids.length),
                  style: context.typography.barTitle,
                ),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).maybePop(),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.noteListCancel,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
        if (notebooks.isEmpty)
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: Text(
              l10n.libraryNoNotebooks,
              style: context.typography.secondary,
            ),
          ),
        if (notebooks.isNotEmpty)
          KansoSheetGroup(
            children: <Widget>[
              for (final notebook in notebooks)
                KansoCheckRow(
                  label: notebook.name,
                  icon: notebook.locked ? KansoIcons.lock : KansoIcons.notebook,
                  indent: notebook.depth * KansoSpacing.md,
                  checked: holding.length == 1 && holding.single == notebook.id,
                  onTap: notebook.locked
                      ? null
                      : () {
                          bloc.add(
                            NoteListEvent.changeRequested(
                              NoteChange.move(notebookId: notebook.id),
                              ids,
                            ),
                          );
                          Navigator.of(context).maybePop();
                        },
                ),
            ],
          ),
      ],
    );
  }
}
