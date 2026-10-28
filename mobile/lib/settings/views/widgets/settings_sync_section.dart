// Kanso — settings_sync_section.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/sync_labels.dart';
import '../../../core/utils/time_words.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../state/sync_settings/sync_settings_bloc.dart';
import 'sync_interval_sheet.dart';

/// Settings › Sync: with no account a row that signs in; with an account
/// that is signed in but locked a row that opens its notes on this device;
/// with one signed in the account's row (its name, to its page), the
/// desktop's status line, Sync now, Sync automatically and Sync every. It
/// says under it that the notes are not in device backups.
///
/// It needs the app's `AccountBloc`, `SyncStatusBloc` and
/// `SyncSettingsBloc`; the settings are read again when an account signs in,
/// since signing in turns sync on.
class SettingsSyncSection extends StatelessWidget {
  const SettingsSyncSection({required this.iconInset, super.key});

  /// How far a row's hairline is inset: past its icon.
  final double iconInset;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocListener<AccountBloc, AccountBlocState>(
      listenWhen: (before, now) => before.signedIn != now.signedIn,
      listener: (context, account) => context.read<SyncSettingsBloc>().add(
        const SyncSettingsEvent.started(),
      ),
      child: BlocBuilder<AccountBloc, AccountBlocState>(
        builder: (context, account) {
          return BlocBuilder<SyncStatusBloc, SyncStatusState>(
            builder: (context, sync) {
              final locked = account.locked;
              final username = account.username;
              return KansoGroupedSection(
                title: l10n.settingsSync,
                footer: l10n.settingsSyncBackupFooter,
                dividerIndent: iconInset,
                children: <Widget>[
                  if (locked != null)
                    KansoSettingsRow(
                      icon: KansoIcons.lock,
                      label: l10n.settingsSyncOpenNotes,
                      chevron: true,
                      onTap: () => context.push(KansoRoutes.settingsUnlock),
                    )
                  else if (account.signedIn && username != null) ...<Widget>[
                    KansoSettingsRow(
                      icon: KansoIcons.user,
                      label: username,
                      value: l10n.settingsSyncAccount,
                      chevron: true,
                      onTap: () => context.push(KansoRoutes.settingsAccount),
                    ),
                    _SyncStatusRow(status: sync.status),
                    KansoSettingsRow(
                      icon: KansoIcons.refreshCw,
                      label: l10n.settingsSyncNow,
                      value: sync.busy ? l10n.settingsSyncSyncing : null,
                      onTap: sync.busy
                          ? null
                          : () => context.read<SyncStatusBloc>().add(
                              const SyncStatusEvent.syncRequested(),
                            ),
                    ),
                    const _SyncAutomaticallyRow(),
                    const _SyncEveryRow(),
                  ] else
                    KansoSettingsRow(
                      icon: KansoIcons.cloud,
                      label: l10n.settingsSyncSignIn,
                      chevron: true,
                      onTap: () => context.push(KansoRoutes.settingsSignIn),
                    ),
                ],
              );
            },
          );
        },
      ),
    );
  }
}

/// The desktop's status line: what sync is doing, and what it means.
class _SyncStatusRow extends StatelessWidget {
  const _SyncStatusRow({required this.status});

  final SyncStatus? status;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    // The settings shape the line ("every 5 minutes", "Sync off").
    context.watch<SyncSettingsBloc>();
    final status = this.status;
    final now = clock.now();
    final last = status?.lastSyncedMs;
    final resume = status?.resumeAtMs;
    final line = status == null
        ? null
        : context.read<SyncSettingsBloc>().statusLine(
            status,
            when: last == null ? '' : editedLabel(l10n, last, now: now),
            resume: resume == null ? '' : resumeLabel(l10n, resume, now: now),
          );
    if (line == null) {
      return KansoSettingsRow(
        icon: KansoIcons.cloud,
        label: l10n.settingsSyncStatus,
        value: l10n.syncStatusName(status),
      );
    }
    return KansoDetailRow(
      icon: switch (line.tone) {
        SyncTone.good => KansoIcons.circleCheck,
        SyncTone.busy => KansoIcons.refreshCw,
        SyncTone.quiet => KansoIcons.circlePause,
        SyncTone.warning => KansoIcons.circleX,
      },
      title: line.label,
      subtitle: line.detail,
    );
  }
}

/// Sync automatically: whether Kanso syncs on its own.
class _SyncAutomaticallyRow extends StatelessWidget {
  const _SyncAutomaticallyRow();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final settings = context.select<SyncSettingsBloc, SyncSettingsRow?>(
      (bloc) => bloc.state.settings,
    );
    void set({required bool on}) => context.read<SyncSettingsBloc>().add(
      SyncSettingsEvent.enabledChanged(enabled: on),
    );
    return Semantics(
      toggled: settings?.enabled ?? false,
      label: l10n.settingsSyncAutomatically,
      excludeSemantics: true,
      child: KansoDetailRow(
        icon: KansoIcons.refreshCw,
        title: l10n.settingsSyncAutomatically,
        subtitle: l10n.settingsSyncAutomaticallyDetail,
        onTap: settings == null ? null : () => set(on: !settings.enabled),
        trailing: KansoSwitch(
          value: settings?.enabled ?? false,
          onChanged: settings == null ? null : (on) => set(on: on),
        ),
      ),
    );
  }
}

/// Sync every: how often Kanso syncs on its own, chosen in a sheet.
class _SyncEveryRow extends StatelessWidget {
  const _SyncEveryRow();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final state = context.watch<SyncSettingsBloc>().state;
    final interval = state.interval;
    return KansoSettingsRow(
      icon: KansoIcons.clock,
      label: l10n.settingsSyncEvery,
      value: interval?.label,
      chevron: true,
      onTap: state.settings == null
          ? null
          : () => SyncIntervalSheet.show(
              context,
              options: state.options,
              minutes: interval?.minutes,
              onChosen: (minutes) => context.read<SyncSettingsBloc>().add(
                SyncSettingsEvent.intervalChosen(minutes),
              ),
            ),
    );
  }
}
