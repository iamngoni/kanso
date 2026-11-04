// Kanso — general_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../state/default_notebook/default_notebook_bloc.dart';
import 'settings_category_pane.dart';
import 'word_goal_default_section.dart';

/// Settings › General: the notebook new notes go in, importing notes, and
/// the writing defaults (focus mode in Live, the default word goal). It
/// needs a `WordGoalSettingsBloc` above it.
class GeneralSettingsPane extends StatelessWidget {
  const GeneralSettingsPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SettingsCategoryPane(
      title: l10n.settingsGeneral,
      sections: <Widget>[
        BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
          builder: (context, index) {
            return BlocBuilder<DefaultNotebookBloc, DefaultNotebookState>(
              builder: (context, chosen) {
                final id = chosen.notebookId;
                final book = id == null ? null : index.notebook(id);
                return KansoGroupedSection(
                  dividerIndent: SettingsCategoryPane.iconInset,
                  children: <Widget>[
                    KansoSettingsRow(
                      icon: KansoIcons.notebook,
                      label: l10n.settingsDefaultNotebook,
                      // A notebook that is gone or locked is not the one
                      // new notes go in.
                      value: book == null || book.locked
                          ? l10n.defaultNotebookFirst
                          : book.name,
                      chevron: true,
                      onTap: () => context.push(KansoRoutes.settingsNotebook),
                    ),
                    KansoSettingsRow(
                      icon: KansoIcons.download,
                      label: l10n.importTitle,
                      chevron: true,
                      onTap: () => context.push(KansoRoutes.import),
                    ),
                  ],
                );
              },
            );
          },
        ),
        const WordGoalDefaultSection(),
      ],
    );
  }
}
