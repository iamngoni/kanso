// Kanso: import_kind_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';

/// The sheet that asks whether to choose a file or a folder, for a source
/// that reads either.
class ImportKindSheet extends StatelessWidget {
  const ImportKindSheet({required this.source, super.key});

  final ImportSource source;

  /// Shows the sheet; completes with `true` for a folder, `false` for a
  /// file, or `null` when it is closed without a choice.
  static Future<bool?> open(BuildContext context, ImportSource source) {
    return showKansoBottomSheet<bool>(
      context,
      builder: (_) => ImportKindSheet(source: source),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.md,
            KansoSpacing.md,
            0,
          ),
          child: Text(
            l10n.importPickTitle(source.label),
            style: context.typography.heading2,
          ),
        ),
        KansoSheetGroup(
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.fileText,
              label: l10n.importPickFile,
              chevron: true,
              onTap: () => Navigator.of(context).pop(false),
            ),
            KansoSettingsRow(
              icon: KansoIcons.notebook,
              label: l10n.importPickFolder,
              chevron: true,
              onTap: () => Navigator.of(context).pop(true),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.md),
      ],
    );
  }
}
