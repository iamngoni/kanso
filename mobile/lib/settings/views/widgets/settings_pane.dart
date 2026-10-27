// Kanso — settings_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/font_labels.dart';
import '../../../core/utils/sync_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../state/about/about_bloc.dart';
import '../../state/default_notebook/default_notebook_bloc.dart';

/// Settings: a close button, a centred title and rows grouped in rounded
/// sections, each with an icon, a label and its current value.
///
/// Only what works is here: how the app looks (themes, fonts, text size),
/// the state of sync with the note that the notes are not in device backups,
/// and what the app is (its version, its website, its licences). It needs
/// the app's `AppearanceBloc` and `SyncStatusBloc` and an [AboutBloc] above
/// it.
class SettingsPane extends StatelessWidget {
  const SettingsPane({super.key});

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
            leading: KansoIconButton(
              icon: KansoIcons.close,
              semanticLabel: l10n.settingsClose,
              onPressed: () => context.pop(),
            ),
            title: l10n.settingsTitle,
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
                BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
                  builder: (context, index) {
                    return BlocBuilder<
                      DefaultNotebookBloc,
                      DefaultNotebookState
                    >(
                      builder: (context, chosen) {
                        final id = chosen.notebookId;
                        final book = id == null ? null : index.notebook(id);
                        return KansoGroupedSection(
                          title: l10n.settingsGeneral,
                          dividerIndent: iconInset,
                          children: <Widget>[
                            KansoSettingsRow(
                              icon: KansoIcons.notebook,
                              label: l10n.settingsDefaultNotebook,
                              // A notebook that is gone or locked is not the
                              // one new notes go in.
                              value: book == null || book.locked
                                  ? l10n.defaultNotebookFirst
                                  : book.name,
                              chevron: true,
                              onTap: () =>
                                  context.push(KansoRoutes.settingsNotebook),
                            ),
                          ],
                        );
                      },
                    );
                  },
                ),
                const SizedBox(height: KansoSpacing.lg),
                BlocBuilder<AppearanceBloc, AppearanceState>(
                  builder: (context, appearance) {
                    final theme =
                        appearance.selection.followSystem &&
                            appearance.light.id != appearance.dark.id
                        ? l10n.settingsThemeBoth(
                            appearance.light.name,
                            appearance.dark.name,
                          )
                        : appearance.light.name;
                    return KansoGroupedSection(
                      title: l10n.settingsAppearance,
                      dividerIndent: iconInset,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.palette,
                          label: l10n.settingsTheme,
                          value: theme,
                          chevron: true,
                          onTap: () => context.push(KansoRoutes.settingsTheme),
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
                          label: l10n.settingsTextSize,
                          value: l10n.textSizeName(appearance.textSize),
                          chevron: true,
                          onTap: () =>
                              context.push(KansoRoutes.fonts(atSize: true)),
                        ),
                      ],
                    );
                  },
                ),
                const SizedBox(height: KansoSpacing.lg),
                BlocBuilder<SyncStatusBloc, SyncStatusState>(
                  builder: (context, sync) {
                    return KansoGroupedSection(
                      title: l10n.settingsSync,
                      footer: l10n.settingsSyncBackupFooter,
                      dividerIndent: iconInset,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.cloud,
                          label: l10n.settingsSyncStatus,
                          value: l10n.syncStatusName(sync.status),
                        ),
                      ],
                    );
                  },
                ),
                const SizedBox(height: KansoSpacing.lg),
                KansoGroupedSection(
                  title: l10n.settingsPrivacy,
                  footer: l10n.settingsPrivacyFooter,
                  dividerIndent: iconInset,
                  children: <Widget>[
                    KansoSettingsRow(
                      icon: KansoIcons.shield,
                      label: l10n.crashReportsTitle,
                      chevron: true,
                      onTap: () =>
                          context.push(KansoRoutes.settingsCrashReports),
                    ),
                  ],
                ),
                const SizedBox(height: KansoSpacing.lg),
                BlocBuilder<AboutBloc, AboutState>(
                  builder: (context, about) {
                    final version = about.version;
                    return KansoGroupedSection(
                      title: l10n.settingsAbout,
                      dividerIndent: iconInset,
                      children: <Widget>[
                        KansoSettingsRow(
                          icon: KansoIcons.info,
                          label: l10n.settingsVersion,
                          value:
                              version == null ||
                                  about.status != LoadStatus.ready
                              ? null
                              : version.patch == null
                              ? l10n.settingsVersionValue(
                                  version.version,
                                  version.build,
                                )
                              : l10n.settingsVersionPatchValue(
                                  version.version,
                                  version.build,
                                  version.patch!,
                                ),
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.globe,
                          label: l10n.settingsWebsite,
                          external: true,
                          onTap: () => context.read<AboutBloc>().add(
                            const AboutEvent.websiteOpened(),
                          ),
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.fileText,
                          label: l10n.settingsReleaseNotes,
                          external: true,
                          onTap: () => context.read<AboutBloc>().add(
                            const AboutEvent.releaseNotesOpened(),
                          ),
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.circleHelp,
                          label: l10n.settingsHelp,
                          external: true,
                          onTap: () => context.read<AboutBloc>().add(
                            const AboutEvent.helpOpened(),
                          ),
                        ),
                        KansoSettingsRow(
                          icon: KansoIcons.scale,
                          label: l10n.settingsLicences,
                          chevron: true,
                          onTap: () =>
                              context.push(KansoRoutes.settingsLicences),
                        ),
                      ],
                    );
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
