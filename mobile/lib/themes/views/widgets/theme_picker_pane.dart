// Kanso — theme_picker_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import '../../utils/themes_in_use_first.dart';
import 'installed_theme_loading_row.dart';
import 'installed_theme_row.dart';
import 'theme_gallery_section.dart';

/// The theme picker: a switch to follow the system's light and dark mode,
/// the installed themes in a row, each a preview of Markdown in its own
/// colours, with the ones in use outlined and first, and below them the
/// gallery of the community's themes to install.
///
/// Choosing a card is the `AppearanceBloc`'s to apply, and shows at once. It
/// needs a [ThemePickerBloc] and a `ThemeGalleryBloc` above it too.
class ThemePickerPane extends StatelessWidget {
  const ThemePickerPane({super.key});

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
            title: l10n.themesTitle,
          ),
          Expanded(
            child: BlocBuilder<AppearanceBloc, AppearanceState>(
              buildWhen: (before, after) => before.selection != after.selection,
              builder: (context, appearance) {
                final selection = appearance.selection;
                return ListView(
                  padding: EdgeInsets.only(
                    top: KansoSpacing.xs,
                    bottom: bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: KansoSpacing.md,
                      ),
                      child: KansoGroupedSection(
                        footer: selection.followSystem
                            ? l10n.themesFollowSystemOn
                            : l10n.themesFollowSystemOff,
                        children: <Widget>[
                          KansoSettingsRow(
                            icon: KansoIcons.monitor,
                            label: l10n.themesFollowSystem,
                            trailing: KansoSwitch(
                              value: selection.followSystem,
                              onChanged: (follow) =>
                                  context.read<AppearanceBloc>().add(
                                    AppearanceEvent.followSystemChanged(
                                      follow: follow,
                                    ),
                                  ),
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.xs),
                    KansoSectionHeader(label: l10n.themesInstalled),
                    BlocBuilder<ThemePickerBloc, ThemePickerState>(
                      builder: (context, picker) {
                        return switch (picker.status) {
                          LoadStatus.loading =>
                            const InstalledThemeLoadingRow(),
                          LoadStatus.failed => Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: KansoSpacing.md,
                            ),
                            child: KansoMessageCard(
                              title: l10n.themesFailedTitle,
                              message: l10n.themesFailedMessage,
                              actions: <Widget>[
                                KansoButton(
                                  label: l10n.retry,
                                  onPressed: () => context
                                      .read<ThemePickerBloc>()
                                      .add(const ThemePickerEvent.started()),
                                ),
                              ],
                            ),
                          ),
                          LoadStatus.ready => InstalledThemeRow(
                            themes: themesInUseFirst(picker.themes, selection),
                            selection: selection,
                          ),
                        };
                      },
                    ),
                    const SizedBox(height: KansoSpacing.xs),
                    const ThemeGallerySection(),
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
