// Kanso — palette_choice_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../models/data/palette_choice.dart';

/// A sheet of the values a palette command asks for, with the one the note
/// already has marked. Completes with the value chosen, or null.
class PaletteChoiceSheet extends StatelessWidget {
  const PaletteChoiceSheet({
    required this.title,
    required this.choices,
    super.key,
  });

  final String title;
  final List<PaletteChoice> choices;

  /// Shows the choices over the whole app.
  static Future<String?> show(
    BuildContext context, {
    required String title,
    required List<PaletteChoice> choices,
  }) {
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => PaletteChoiceSheet(title: title, choices: choices),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(title, style: context.typography.barTitle),
        ),
        Flexible(
          child: ListView(
            shrinkWrap: true,
            padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
            children: <Widget>[
              KansoSheetGroup(
                dividerIndent: KansoSettingsRow.iconInset,
                children: <Widget>[
                  for (final choice in choices)
                    KansoSettingsRow(
                      icon: choice.checked
                          ? KansoIcons.circleCheck
                          : KansoIcons.circleDot,
                      label: choice.label,
                      onTap: () => Navigator.of(context).pop(choice.value),
                    ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }
}
