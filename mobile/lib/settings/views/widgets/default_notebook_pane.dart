// Kanso — default_notebook_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/default_notebook/default_notebook_bloc.dart';

/// Settings › General › Default notebook: "First notebook", then the
/// notebooks in the order of the library, the one in use checked. A locked
/// notebook is dimmed and cannot be chosen: a new note could not go in it
/// unless it was open.
///
/// Where the saved choice is gone or locked, "First notebook" is the one
/// checked, which is what new notes then do.
class DefaultNotebookPane extends StatelessWidget {
  const DefaultNotebookPane({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    final bloc = context.read<DefaultNotebookBloc>();
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.settingsDefaultNotebook,
          ),
          Expanded(
            child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
              builder: (context, index) {
                return BlocBuilder<DefaultNotebookBloc, DefaultNotebookState>(
                  builder: (context, chosen) {
                    final notebooks = <NotebookRow>[
                      for (final book in index.notebooks)
                        if (book.role != NotebookRole.templates) book,
                    ];
                    final inUse = notebooks
                        .where((book) => !book.locked)
                        .map((book) => book.id)
                        .contains(chosen.notebookId);
                    return ListView(
                      padding: EdgeInsets.fromLTRB(
                        KansoSpacing.md,
                        KansoSpacing.xs,
                        KansoSpacing.md,
                        bottom + KansoSpacing.lg,
                      ),
                      children: <Widget>[
                        KansoGroupedSection(
                          footer: l10n.defaultNotebookFooter,
                          children: <Widget>[
                            KansoCheckRow(
                              label: l10n.defaultNotebookFirst,
                              icon: KansoIcons.notebook,
                              checked: !inUse,
                              onTap: () => bloc.add(
                                const DefaultNotebookEvent.chosen(null),
                              ),
                            ),
                            for (final book in notebooks)
                              KansoCheckRow(
                                label: book.name,
                                icon: book.locked
                                    ? KansoIcons.lock
                                    : KansoIcons.notebook,
                                indent: book.depth * KansoSpacing.md,
                                checked: inUse && book.id == chosen.notebookId,
                                onTap: book.locked
                                    ? null
                                    : () => bloc.add(
                                        DefaultNotebookEvent.chosen(book.id),
                                      ),
                              ),
                          ],
                        ),
                      ],
                    );
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
