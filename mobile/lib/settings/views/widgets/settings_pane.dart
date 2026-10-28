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
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/font_labels.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_large_title.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../push/views/widgets/push_setting_notice_host.dart';
import '../../../reminders/views/widgets/reminders_section.dart';
import '../../../templates/views/widgets/daily_template_section.dart';
import '../../state/about/about_bloc.dart';
import '../../state/default_notebook/default_notebook_bloc.dart';
import '../../state/discovery/discovery_settings_bloc.dart';
import 'discovery_setting_switch_row.dart';
import 'discovery_status_row.dart';
import 'locked_notes_section.dart';
import 'settings_sync_section.dart';
import 'word_goal_default_section.dart';

/// Settings: a close button, a centred title and rows grouped in rounded
/// sections, each with an icon, a label and its current value.
///
/// Only what works is here: how the app looks (themes, icons, fonts, text
/// size), the account and the state of sync with the note that the notes
/// are not in device backups, and what the app is (its version, its website,
/// its licences). It needs the app's `AppearanceBloc`, `AccountBloc` and
/// `SyncStatusBloc` and an [AboutBloc] above it.
class SettingsPane extends StatelessWidget {
  const SettingsPane({super.key});

  /// How far a row's hairline is inset: past its icon.
  static const double iconInset = 48;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return PushSettingNoticeHost(
      child: ColoredBox(
        color: colors.app,
        child: KansoLargeTitleScroll(
          builder: (context, collapsed) => Column(
            children: <Widget>[
              KansoTopBar(
                backgroundColor: colors.app,
                titleShown: collapsed,
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
                    0,
                    KansoSpacing.md,
                    bottom + KansoSpacing.lg,
                  ),
                  children: <Widget>[
                    KansoLargeTitle(
                      title: l10n.settingsTitle,
                      padding: const EdgeInsets.only(bottom: KansoSpacing.md),
                    ),
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
                                  // A notebook that is gone or locked is
                                  // not the one new notes go in.
                                  value: book == null || book.locked
                                      ? l10n.defaultNotebookFirst
                                      : book.name,
                                  chevron: true,
                                  onTap: () => context.push(
                                    KansoRoutes.settingsNotebook,
                                  ),
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
                    const SizedBox(height: KansoSpacing.lg),
                    const WordGoalDefaultSection(),
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
                              onTap: () =>
                                  context.push(KansoRoutes.settingsTheme),
                            ),
                            KansoSettingsRow(
                              icon: KansoIcons.shapes,
                              label: l10n.settingsIcons,
                              value: appearance.iconSet.name,
                              chevron: true,
                              onTap: () =>
                                  context.push(KansoRoutes.settingsIcons),
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
                    const SettingsSyncSection(iconInset: iconInset),
                    const SizedBox(height: KansoSpacing.lg),
                    const DailyTemplateSection(dividerIndent: iconInset),
                    const SizedBox(height: KansoSpacing.lg),
                    const RemindersSection(dividerIndent: iconInset),
                    const SizedBox(height: KansoSpacing.lg),
                    BlocBuilder<DiscoverySettingsBloc, DiscoverySettingsState>(
                      builder: (context, discovery) => KansoGroupedSection(
                        title: l10n.settingsDiscovery,
                        dividerIndent: iconInset,
                        children: <Widget>[
                          DiscoverySettingSwitchRow(
                            title: l10n.discoveryRelated,
                            detail: l10n.discoveryRelatedDetail,
                            value: discovery.relatedEnabled,
                            onChanged: discovery.relatedBusy
                                ? null
                                : (enabled) =>
                                      context.read<DiscoverySettingsBloc>().add(
                                        DiscoverySettingsEvent.relatedChanged(
                                          enabled: enabled,
                                        ),
                                      ),
                          ),
                          if (discovery.relatedEnabled)
                            DiscoveryStatusRow(
                              status: discovery.related,
                              onRetry: () {
                                context.read<DiscoverySettingsBloc>().add(
                                  const RelatedRetryRequestedEvent(),
                                );
                              },
                            ),
                          DiscoverySettingSwitchRow(
                            title: l10n.discoverySearchImages,
                            detail: l10n.discoverySearchImagesDetail,
                            value: discovery.ocrEnabled,
                            onChanged: (enabled) =>
                                context.read<DiscoverySettingsBloc>().add(
                                  DiscoverySettingsEvent.ocrChanged(
                                    enabled: enabled,
                                  ),
                                ),
                          ),
                          if (discovery.ocrEnabled)
                            Padding(
                              padding: const EdgeInsets.fromLTRB(
                                KansoSpacing.md,
                                0,
                                KansoSpacing.md,
                                KansoSpacing.sm,
                              ),
                              child: Text(
                                l10n.discoveryReadingImages(
                                  discovery.ocrWaiting,
                                ),
                                style: context.typography.secondary,
                              ),
                            ),
                        ],
                      ),
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
                        KansoSettingsRow(
                          icon: KansoIcons.databaseBackup,
                          label: l10n.backupsTitle,
                          value: l10n.backupsSchedule,
                          chevron: true,
                          onTap: () =>
                              context.push(KansoRoutes.settingsBackups),
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    const LockedNotesSection(dividerIndent: iconInset),
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
        ),
      ),
    );
  }
}
