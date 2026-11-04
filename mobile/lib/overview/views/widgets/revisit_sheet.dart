// Kanso — revisit_sheet.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/overview_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../state/overview_cubit.dart';

/// "Revisit in…": six days to bring a note back on Overview, each with its
/// date. Setting one is not an edit of the note.
Future<RevisitChoice?> showRevisitSheet(BuildContext context) {
  final options = getIt<DigestRepository>().revisitOptions();
  return showKansoBottomSheet<RevisitChoice>(
    context,
    builder: (sheet) {
      final l10n = sheet.l10n;
      return Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SheetHeader(
            title: l10n.overviewRevisitTitle,
            actionLabel: l10n.overviewCancel,
          ),
          KansoSheetGroup(
            dividerIndent: KansoSpacing.md + 28,
            children: <Widget>[
              for (final option in options)
                KansoSettingsRow(
                  icon: KansoIcons.clock,
                  label: option.label,
                  value: option.dayLabel,
                  onTap: () => Navigator.of(sheet).pop(option.choice),
                ),
            ],
          ),
          const SizedBox(height: KansoSpacing.md),
        ],
      );
    },
  );
}

/// Asks which day, and answers with it in words, or null when dismissed.
Future<OverviewRevisit?> pickRevisit(BuildContext context) async {
  final options = getIt<DigestRepository>().revisitOptions();
  final choice = await showRevisitSheet(context);
  if (choice == null) {
    return null;
  }
  final option = options.firstWhere((it) => it.choice == choice);
  return OverviewRevisit(option.choice, option.dayLabel);
}
