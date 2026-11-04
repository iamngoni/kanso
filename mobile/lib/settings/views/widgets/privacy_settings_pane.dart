// Kanso — privacy_settings_pane.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import 'locked_notes_section.dart';
import 'settings_category_pane.dart';

/// Settings › Privacy and security: crash reports, device backups and, once
/// a lock password exists, locked notes. It needs a `BiometricsBloc` above
/// it.
class PrivacySettingsPane extends StatelessWidget {
  const PrivacySettingsPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SettingsCategoryPane(
      title: l10n.settingsPrivacySecurity,
      sections: <Widget>[
        KansoGroupedSection(
          title: l10n.settingsPrivacy,
          footer: l10n.settingsPrivacyFooter,
          dividerIndent: SettingsCategoryPane.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.shield,
              label: l10n.crashReportsTitle,
              chevron: true,
              onTap: () => context.push(KansoRoutes.settingsCrashReports),
            ),
            KansoSettingsRow(
              icon: KansoIcons.databaseBackup,
              label: l10n.backupsTitle,
              value: l10n.backupsSchedule,
              chevron: true,
              onTap: () => context.push(KansoRoutes.settingsBackups),
            ),
          ],
        ),
        // Last: it brings its own space under it, and is not there until
        // a lock password exists.
        const LockedNotesSection(dividerIndent: SettingsCategoryPane.iconInset),
      ],
    );
  }
}
