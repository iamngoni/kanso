// Kanso — theme_picker_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/data/theme_selection.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/data/theme_card_kind.dart';
import '../../models/data/theme_preview.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import '../../utils/themes_in_use_first.dart';
import 'installed_theme_loading_row.dart';
import 'installed_theme_row.dart';
import 'theme_files_section.dart';
import 'theme_gallery_section.dart';
import 'theme_picker_section.dart';

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
                    BlocBuilder<ThemePickerBloc, ThemePickerState>(
                      buildWhen: (before, after) =>
                          before.status != after.status ||
                          before.themes != after.themes,
                      builder: (context, picker) {
                        return switch (picker.status) {
                          LoadStatus.loading => Column(
                            children: <Widget>[
                              for (final kind
                                  in ThemeCardKind.values) ...<Widget>[
                                ThemePickerSection(
                                  title: _title(l10n, kind),
                                  detail: _detail(l10n, kind),
                                  child: const InstalledThemeLoadingRow(),
                                ),
                              ],
                            ],
                          ),
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
                          LoadStatus.ready => _Grids(
                            themes: picker.themes,
                            selection: selection,
                          ),
                        };
                      },
                    ),
                    const SizedBox(height: KansoSpacing.xs),
                    const ThemeFilesSection(),
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

String _title(AppLocalizations l10n, ThemeCardKind kind) => switch (kind) {
  ThemeCardKind.ui => l10n.themesSectionTheme,
  ThemeCardKind.syntax => l10n.themesSectionSyntax,
  ThemeCardKind.preview => l10n.themesSectionPreview,
};

String _detail(AppLocalizations l10n, ThemeCardKind kind) => switch (kind) {
  ThemeCardKind.ui => l10n.themesSectionThemeDetail,
  ThemeCardKind.syntax => l10n.themesSectionSyntaxDetail,
  ThemeCardKind.preview => l10n.themesSectionPreviewDetail,
};

/// The three rows of cards: the interface's theme, the Markdown source's
/// and the preview's.
class _Grids extends StatelessWidget {
  const _Grids({required this.themes, required this.selection});

  final List<ThemePreview> themes;
  final ThemeSelection selection;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final dark = Theme.brightnessOf(context) == Brightness.dark;
    final shownId = selection.uiFor(systemDark: dark);
    final shown = themes.isEmpty
        ? null
        : themes.firstWhere(
            (theme) => theme.id == shownId,
            orElse: () => themes.first,
          );
    return Column(
      children: <Widget>[
        for (final kind in ThemeCardKind.values) ...<Widget>[
          ThemePickerSection(
            title: _title(l10n, kind),
            detail: _detail(l10n, kind),
            child: InstalledThemeRow(
              kind: kind,
              themes: kind == ThemeCardKind.ui
                  ? themesInUseFirst(themes, selection)
                  : themes,
              selection: selection,
              sameAs: shown,
            ),
          ),
          const SizedBox(height: KansoSpacing.sm),
        ],
      ],
    );
  }
}
