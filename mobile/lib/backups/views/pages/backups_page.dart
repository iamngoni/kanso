// Kanso — backups_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../models/data/backups_notice.dart';
import '../../models/data/backups_problem.dart';
import '../../state/backups/backups_bloc.dart';
import '../widgets/backup_row.dart';

/// Lists the daily and on-request copies of this device's library.
class BackupsPage extends StatelessWidget {
  const BackupsPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Scaffold(
      backgroundColor: context.colors.app,
      body: Column(
        children: <Widget>[
          KansoTopBar(
            title: l10n.backupsTitle,
            leading: KansoIconButton(
              icon: KansoIcons.close,
              semanticLabel: l10n.settingsClose,
              onPressed: () => context.pop(),
            ),
          ),
          Expanded(
            child: BlocConsumer<BackupsBloc, BackupsState>(
              listener: (context, state) {
                final notice = state.notice;
                if (notice != null) {
                  final message = notice == BackupsNotice.madeNow
                      ? l10n.backupsNoticeMadeNow
                      : l10n.backupsNoticeDeleted;
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text(message)),
                  );
                }
              },
              builder: (context, state) => ListView(
                padding: const EdgeInsets.all(KansoSpacing.md),
                children: <Widget>[
                  Text(
                    l10n.backupsSummary,
                    style: context.typography.secondary,
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  KansoButton(
                    label: state.working
                        ? l10n.backupsBackingUp
                        : l10n.backupsBackUpNow,
                    icon: KansoIcons.databaseBackup,
                    onPressed: state.working
                        ? null
                        : () => context.read<BackupsBloc>().add(
                            const BackupsEvent.backUpNow(),
                          ),
                  ),
                  if (state.problem != null)
                    Padding(
                      padding: const EdgeInsets.only(top: KansoSpacing.sm),
                      child: Text(switch (state.problem!) {
                        BackupsProblem.dailyFailed => l10n.backupsProblemDaily,
                        BackupsProblem.backUpFailed =>
                          l10n.backupsProblemBackUp,
                        BackupsProblem.deleteFailed =>
                          l10n.backupsProblemDelete,
                        BackupsProblem.listFailed => l10n.backupsProblemList,
                      }, style: TextStyle(color: context.colors.danger)),
                    ),
                  const SizedBox(height: KansoSpacing.lg),
                  Text(
                    l10n.backupsListHeader,
                    style: context.typography.heading2,
                  ),
                  if (state.status == LoadStatus.loading)
                    Semantics(
                      label: l10n.backupsLoading,
                      child: const Padding(
                        padding: EdgeInsets.all(KansoSpacing.md),
                        child: LinearProgressIndicator(),
                      ),
                    )
                  else if (state.status == LoadStatus.failed)
                    KansoMessageCard(
                      icon: KansoIcons.databaseBackup,
                      title: l10n.backupsFailedTitle,
                      message: l10n.backupsFailedMessage,
                      actions: <Widget>[
                        KansoButton(
                          label: l10n.retry,
                          onPressed: () => context.read<BackupsBloc>().add(
                            const BackupsEvent.refreshed(),
                          ),
                        ),
                      ],
                    )
                  else if (state.backups.isEmpty)
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        vertical: KansoSpacing.lg,
                      ),
                      child: KansoMessageCard(
                        icon: KansoIcons.databaseBackup,
                        title: l10n.backupsEmptyTitle,
                        message: l10n.backupsEmptyMessage,
                        actions: const <Widget>[],
                      ),
                    )
                  else
                    for (final backup in state.backups)
                      BackupRow(
                        backup: backup,
                        deleting: state.deleting.contains(backup.path),
                      ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
