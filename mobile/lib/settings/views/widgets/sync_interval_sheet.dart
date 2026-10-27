// Kanso — sync_interval_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';

/// How often sync runs: the desktop's choices, with a check on the one in
/// use. A sheet; choosing closes it.
class SyncIntervalSheet extends StatelessWidget {
  const SyncIntervalSheet({
    required this.options,
    required this.minutes,
    required this.onChosen,
    super.key,
  });

  final List<SyncIntervalOption> options;

  /// The choice in use, in minutes.
  final int? minutes;

  /// Runs with the chosen minutes once the sheet has closed.
  final ValueChanged<int> onChosen;

  static Future<void> show(
    BuildContext context, {
    required List<SyncIntervalOption> options,
    required int? minutes,
    required ValueChanged<int> onChosen,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (context) => SyncIntervalSheet(
        options: options,
        minutes: minutes,
        onChosen: onChosen,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            context.l10n.settingsSyncEveryTitle,
            style: context.typography.heading2,
          ),
        ),
        for (final option in options)
          KansoSettingsRow(
            label: option.label,
            trailing: option.minutes == minutes
                ? KansoIcon(KansoIcons.check, color: colors.accent)
                : null,
            onTap: () {
              Navigator.of(context).pop();
              onChosen(option.minutes);
            },
          ),
        const SizedBox(height: KansoSpacing.xs),
      ],
    );
  }
}
