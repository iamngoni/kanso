// Kanso — appearance_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart' show DiagramRenderer;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/font_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import 'settings_category_pane.dart';

/// Settings › Appearance: the themes, icons, fonts and text sizes, the
/// preview's text size, and which renderer draws diagrams.
class AppearanceSettingsPane extends StatelessWidget {
  const AppearanceSettingsPane({super.key});

  /// What the Theme row says: both themes when the app follows the system
  /// between two, otherwise the one in use.
  static String themeSummary(BuildContext context, AppearanceState appearance) {
    final followsTwo =
        appearance.selection.followSystem &&
        appearance.light.id != appearance.dark.id;
    return followsTwo
        ? context.l10n.settingsThemeBoth(
            appearance.light.name,
            appearance.dark.name,
          )
        : appearance.light.name;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<AppearanceBloc, AppearanceState>(
      builder: (context, appearance) {
        return SettingsCategoryPane(
          title: l10n.settingsAppearance,
          sections: <Widget>[
            KansoGroupedSection(
              dividerIndent: SettingsCategoryPane.iconInset,
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.palette,
                  label: l10n.settingsTheme,
                  value: themeSummary(context, appearance),
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.settingsTheme),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.shapes,
                  label: l10n.settingsIcons,
                  value: appearance.iconSet.name,
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.settingsIcons),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.download,
                  label: l10n.settingsIconThemes,
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.settingsIconThemes),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.typeface,
                  label: l10n.settingsFonts,
                  value: l10n.fontsSummary(appearance.fonts),
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.fonts()),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.textSize,
                  label: l10n.settingsInterfaceTextSize,
                  value: l10n.textScaleName(appearance.interfaceTextSize),
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.fonts(atSize: true)),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.textSize,
                  label: l10n.settingsTextSize,
                  value: l10n.textScaleName(appearance.textSize),
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.fonts(atSize: true)),
                ),
              ],
            ),
            KansoGroupedSection(
              title: l10n.settingsPreview,
              footer: l10n.fontsPreviewTextSizeFooter,
              dividerIndent: SettingsCategoryPane.iconInset,
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.textSize,
                  label: l10n.fontsPreviewTextSizeSection,
                  value: l10n.textScaleName(appearance.previewTextSize),
                  chevron: true,
                  onTap: () => context.push(KansoRoutes.fonts(atSize: true)),
                ),
              ],
            ),
            KansoGroupedSection(
              title: l10n.settingsDiagrams,
              footer: l10n.settingsDiagramsDetail,
              dividerIndent: SettingsCategoryPane.iconInset,
              children: <Widget>[
                Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: KansoSegmented<DiagramRenderer>(
                    expanded: true,
                    selected: appearance.diagramRenderer,
                    segments: <KansoSegment<DiagramRenderer>>[
                      KansoSegment<DiagramRenderer>(
                        value: DiagramRenderer.pele,
                        label: l10n.diagramRendererPele,
                      ),
                      KansoSegment<DiagramRenderer>(
                        value: DiagramRenderer.kanso,
                        label: l10n.diagramRendererKanso,
                      ),
                    ],
                    onChanged: (value) => context.read<AppearanceBloc>().add(
                      AppearanceEvent.diagramRendererChanged(value),
                    ),
                  ),
                ),
              ],
            ),
          ],
        );
      },
    );
  }
}
