// Kanso — settings_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_large_title.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../feedback/views/widgets/feedback_settings_section.dart';
import '../../../push/views/widgets/push_setting_notice_host.dart';
import 'settings_sync_section.dart';

/// Settings: a close button, a large title, the account and sync, and then
/// a row for each category of settings, which opens it on a page of its own
/// (General, Appearance, Daily notes and reminders, Discovery, the AI and
/// voice pages, Privacy and security, About). Feedback stays here too, as
/// something to do rather than a setting.
///
/// Sync stays here, since it is what is looked at most. It needs the app's
/// `AccountBloc`, `SyncStatusBloc`, `SyncSettingsBloc` and `AiStatusBloc`,
/// and a `PushSettingBloc` and a `MyFeedbackBloc` above it.
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
                    const SettingsSyncSection(iconInset: iconInset),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      dividerIndent: iconInset,
                      children: <Widget>[
                        _CategoryRow(
                          icon: KansoIcons.settings,
                          label: l10n.settingsGeneral,
                          route: KansoRoutes.settingsGeneral,
                        ),
                        _CategoryRow(
                          icon: KansoIcons.palette,
                          label: l10n.settingsAppearance,
                          route: KansoRoutes.settingsAppearance,
                        ),
                        _CategoryRow(
                          icon: KansoIcons.calendarDays,
                          label: l10n.settingsJournal,
                          route: KansoRoutes.settingsJournal,
                        ),
                        _CategoryRow(
                          icon: KansoIcons.calendar,
                          label: l10n.settingsCalendar,
                          route: KansoRoutes.settingsCalendar,
                        ),
                        _CategoryRow(
                          icon: KansoIcons.compass,
                          label: l10n.settingsDiscovery,
                          route: KansoRoutes.settingsDiscovery,
                        ),
                      ],
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    BlocBuilder<AiStatusBloc, AiStatusState>(
                      builder: (context, ai) => KansoGroupedSection(
                        dividerIndent: iconInset,
                        children: <Widget>[
                          KansoSettingsRow(
                            icon: KansoIcons.sparkles,
                            label: l10n.settingsAi,
                            value: ai.enabled
                                ? l10n.settingsAiOn
                                : l10n.settingsAiOff,
                            chevron: true,
                            onTap: () => context.push(KansoRoutes.settingsAi),
                          ),
                          _CategoryRow(
                            icon: KansoIcons.users,
                            label: l10n.peopleTitle,
                            route: KansoRoutes.settingsPeople,
                          ),
                          _CategoryRow(
                            icon: KansoIcons.inbox,
                            label: l10n.triageSettingsOpen,
                            route: KansoRoutes.settingsInbox,
                          ),
                          _CategoryRow(
                            icon: KansoIcons.mic,
                            label: l10n.dictationSettingsTitle,
                            route: KansoRoutes.settingsVoice,
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: KansoSpacing.lg),
                    const FeedbackSettingsSection(dividerIndent: iconInset),
                    const SizedBox(height: KansoSpacing.lg),
                    KansoGroupedSection(
                      dividerIndent: iconInset,
                      children: <Widget>[
                        _CategoryRow(
                          icon: KansoIcons.shieldCheck,
                          label: l10n.settingsPrivacySecurity,
                          route: KansoRoutes.settingsPrivacy,
                        ),
                        _CategoryRow(
                          icon: KansoIcons.info,
                          label: l10n.settingsAbout,
                          route: KansoRoutes.settingsAbout,
                        ),
                      ],
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

/// A row that opens the page at [route].
class _CategoryRow extends StatelessWidget {
  const _CategoryRow({
    required this.icon,
    required this.label,
    required this.route,
  });

  final KansoIcons icon;
  final String label;
  final String route;

  @override
  Widget build(BuildContext context) {
    return KansoSettingsRow(
      icon: icon,
      label: label,
      chevron: true,
      onTap: () => context.push(route),
    );
  }
}
