// Kanso — sketch_size_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';

/// The sizes "Insert sketch" offers, in points: small, medium and large.
const List<(int, int)> sketchSizes = <(int, int)>[
  (320, 240),
  (480, 320),
  (640, 480),
];

/// The sheet that asks how big a new sketch is.
class SketchSizeSheet extends StatelessWidget {
  const SketchSizeSheet({super.key});

  /// Asks; null when the sheet is dismissed.
  static Future<(int, int)?> show(BuildContext context) =>
      showKansoBottomSheet<(int, int)>(
        context,
        builder: (_) => const SketchSizeSheet(),
      );

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final labels = <String>[
      l10n.sketchSizeSmall,
      l10n.sketchSizeMedium,
      l10n.sketchSizeLarge,
    ];
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
          child: Text(
            l10n.sketchSizeTitle,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            for (var i = 0; i < sketchSizes.length; i++)
              KansoSettingsRow(
                icon: KansoIcons.penLine,
                label: labels[i],
                onTap: () => Navigator.of(context).pop(sketchSizes[i]),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}
