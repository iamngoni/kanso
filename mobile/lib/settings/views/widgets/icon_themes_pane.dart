// Kanso — icon_themes_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/icon_themes/icon_themes_bloc.dart';
import 'icon_theme_choice_row.dart';
import 'icon_themes_loading_list.dart';

/// Settings › Icons: the icon themes in a list, each with a row of its icons
/// and who made it, the one in use checked.
///
/// Choosing one is the `AppearanceBloc`'s to apply and redraws every icon in
/// the app at once. It needs an [IconThemesBloc] above it too.
class IconThemesPane extends StatelessWidget {
  const IconThemesPane({super.key});

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
            title: l10n.iconsTitle,
          ),
          Expanded(
            child: ListView(
              padding: EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.xs,
                KansoSpacing.md,
                bottom + KansoSpacing.lg,
              ),
              children: <Widget>[
                BlocBuilder<IconThemesBloc, IconThemesState>(
                  builder: (context, picker) {
                    return switch (picker.status) {
                      LoadStatus.loading => const IconThemesLoadingList(),
                      LoadStatus.failed => Center(
                        child: KansoMessageCard(
                          title: l10n.iconsFailedTitle,
                          message: l10n.iconsFailedMessage,
                          actions: <Widget>[
                            KansoButton(
                              label: l10n.retry,
                              onPressed: () => context
                                  .read<IconThemesBloc>()
                                  .add(const IconThemesEvent.started()),
                            ),
                          ],
                        ),
                      ),
                      LoadStatus.ready when picker.themes.isEmpty => Center(
                        child: KansoMessageCard(
                          title: l10n.iconsEmptyTitle,
                          message: l10n.iconsEmptyMessage,
                          actions: <Widget>[
                            KansoButton(
                              label: l10n.retry,
                              onPressed: () => context
                                  .read<IconThemesBloc>()
                                  .add(const IconThemesEvent.started()),
                            ),
                          ],
                        ),
                      ),
                      LoadStatus.ready =>
                        BlocBuilder<AppearanceBloc, AppearanceState>(
                          buildWhen: (before, after) =>
                              before.iconSet.id != after.iconSet.id,
                          builder: (context, appearance) {
                            return KansoGroupedSection(
                              footer: l10n.iconsFooter,
                              children: <Widget>[
                                for (final theme in picker.themes)
                                  IconThemeChoiceRow(
                                    key: ValueKey<String>(theme.id),
                                    theme: theme,
                                    selected: theme.id == appearance.iconSet.id,
                                    onTap: () =>
                                        context.read<AppearanceBloc>().add(
                                          AppearanceEvent.iconThemePicked(
                                            theme.id,
                                          ),
                                        ),
                                  ),
                              ],
                            );
                          },
                        ),
                    };
                  },
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
