// Kanso — icon_themes_loading_list.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import 'icon_theme_choice_row.dart';

/// Where the icon themes will be while they are read: still rows.
class IconThemesLoadingList extends StatelessWidget {
  const IconThemesLoadingList({super.key});

  /// How many rows are drawn.
  static const int rows = 5;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.iconsLoading,
      container: true,
      child: KansoGroupedSection(
        children: <Widget>[
          for (var i = 0; i < rows; i++)
            const SizedBox(
              height: IconThemeChoiceRow.minHeight,
              child: Padding(
                padding: EdgeInsets.all(KansoSpacing.md),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: <Widget>[
                    KansoSkeleton(width: 120, height: 14),
                    SizedBox(height: KansoSpacing.xs),
                    KansoSkeleton(width: 200, height: 10),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}
