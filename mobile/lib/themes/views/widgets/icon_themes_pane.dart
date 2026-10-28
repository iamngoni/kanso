// Kanso — icon_themes_pane.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/icon_themes/icon_themes_bloc.dart';
import 'icon_theme_row.dart';
import 'theme_import_outcome_row.dart';

/// Settings › Icon themes: the icon themes that were imported or installed
/// from the gallery, and the Import icon theme row, which opens the system
/// file picker. It needs an [IconThemesBloc] above it.
///
/// The list is where the choice of an icon theme will go: each row is an
/// [IconThemeRow], which has room for a check mark, and the bundled icon
/// themes can be listed in front of the imported ones.
class IconThemesPane extends StatelessWidget {
  const IconThemesPane({super.key});

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.app,
      child: Column(
        children: <Widget>[
          KansoTopBar(
            backgroundColor: colors.app,
            bordered: false,
            leading: const ShellBackButton(),
            title: l10n.iconThemesTitle,
          ),
          Expanded(
            child: BlocBuilder<IconThemesBloc, IconThemesState>(
              builder: (context, state) {
                void import() => context.read<IconThemesBloc>().add(
                  const IconThemesEvent.importRequested(),
                );
                final outcome = state.outcome;
                return ListView(
                  padding: EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    switch (state.status) {
                      LoadStatus.loading => const SizedBox.shrink(),
                      LoadStatus.failed => Center(
                        child: KansoMessageCard(
                          title: l10n.iconThemesFailedTitle,
                          message: l10n.iconThemesFailedMessage,
                          actions: <Widget>[
                            KansoButton(
                              label: l10n.retry,
                              onPressed: () =>
                                  context.read<IconThemesBloc>().add(
                                    const IconThemesEvent.started(),
                                  ),
                            ),
                          ],
                        ),
                      ),
                      LoadStatus.ready when state.themes.isEmpty => Padding(
                        padding: const EdgeInsets.symmetric(
                          vertical: KansoSpacing.lg,
                        ),
                        child: Column(
                          children: <Widget>[
                            Text(
                              l10n.iconThemesEmptyTitle,
                              style: context.typography.listTitle,
                            ),
                            const SizedBox(height: KansoSpacing.xxs),
                            Text(
                              l10n.iconThemesEmptyMessage,
                              textAlign: TextAlign.center,
                              style: context.typography.secondary,
                            ),
                          ],
                        ),
                      ),
                      LoadStatus.ready => KansoGroupedSection(
                        title: l10n.iconThemesListSection,
                        dividerIndent: iconInset,
                        children: <Widget>[
                          for (final theme in state.themes)
                            IconThemeRow(
                              key: ValueKey<String>('icon-theme-${theme.id}'),
                              theme: theme,
                            ),
                        ],
                      ),
                    },
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      title: l10n.iconThemesSection,
                      footer: l10n.iconThemesImportDetail,
                      dividerIndent: iconInset,
                      children: <Widget>[
                        KansoSettingsRow(
                          key: const ValueKey<String>('import-icon-theme'),
                          icon: KansoIcons.download,
                          label: l10n.iconThemesImport,
                          trailing: KansoButton(
                            label: state.importing
                                ? l10n.themesImporting
                                : l10n.iconThemesImportButton,
                            variant: KansoButtonVariant.secondary,
                            expanded: false,
                            compact: true,
                            onPressed: state.importing ? null : import,
                          ),
                          onTap: state.importing ? null : import,
                        ),
                        if (outcome != null)
                          ThemeImportOutcomeRow(
                            outcome: outcome,
                            importedLabel: l10n.iconThemesImported,
                            failedLabel: l10n.iconThemesImportFailed,
                          ),
                      ],
                    ),
                  ],
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
