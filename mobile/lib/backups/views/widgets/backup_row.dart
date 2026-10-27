// Kanso — backup_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../state/backups/backups_bloc.dart';

/// A daily copy cannot be deleted; on-request copies can.
class BackupRow extends StatelessWidget {
  const BackupRow({required this.backup, required this.deleting, super.key});

  final BackupFile backup;
  final bool deleting;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final when = DateTime.fromMillisecondsSinceEpoch(backup.madeAtMs).toLocal();
    final size = backup.bytes < 1024
        ? l10n.fileSizeBytes(backup.bytes.toString())
        : backup.bytes < 1024 * 1024
        ? l10n.fileSizeKilobytes((backup.bytes / 1024).toStringAsFixed(1))
        : l10n.fileSizeMegabytes(
            (backup.bytes / (1024 * 1024)).toStringAsFixed(1),
          );
    return ListTile(
      contentPadding: EdgeInsets.zero,
      leading: const KansoIcon(KansoIcons.databaseBackup),
      title: Text(MaterialLocalizations.of(context).formatMediumDate(when)),
      subtitle: Text(
        l10n.backupsKindManualKept(
          backup.manual ? l10n.backupsKindManual : l10n.backupsKindDaily,
          size,
        ),
      ),
      trailing: backup.manual
          ? IconButton(
              tooltip: l10n.backupsDelete,
              onPressed: deleting
                  ? null
                  : () async {
                      final confirmed = await showDialog<bool>(
                        context: context,
                        builder: (context) => AlertDialog(
                          title: Text(l10n.backupsDeleteTitle),
                          content: Text(
                            l10n.backupsDeleteMessage(
                              MaterialLocalizations.of(
                                context,
                              ).formatMediumDate(when),
                            ),
                          ),
                          actions: <Widget>[
                            TextButton(
                              onPressed: () => Navigator.pop(context, false),
                              child: Text(l10n.backupsDeleteCancel),
                            ),
                            TextButton(
                              onPressed: () => Navigator.pop(context, true),
                              child: Text(l10n.backupsDeleteConfirm),
                            ),
                          ],
                        ),
                      );
                      if ((confirmed ?? false) && context.mounted) {
                        context.read<BackupsBloc>().add(
                          BackupsEvent.delete(backup.path),
                        );
                      }
                    },
              icon: deleting
                  ? const SizedBox.square(
                      dimension: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.delete_outline),
            )
          : null,
    );
  }
}
