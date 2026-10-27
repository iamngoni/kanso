// Kanso — kanso_sheet_group.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../configs/kanso_spacing.dart';
import 'kanso_grouped_section.dart';

/// A rounded group of rows in a sheet, inset from its edges, with an
/// optional [title] over it: a sheet's choices drawn as Settings draws its
/// rows.
class KansoSheetGroup extends StatelessWidget {
  const KansoSheetGroup({
    required this.children,
    this.title,
    this.dividerIndent = KansoSpacing.md,
    super.key,
  });

  final String? title;
  final List<Widget> children;

  /// How far the hairlines are inset from the start: past the icon, for
  /// rows that have one.
  final double dividerIndent;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.sm,
        KansoSpacing.md,
        0,
      ),
      child: KansoGroupedSection(
        title: title,
        dividerIndent: dividerIndent,
        children: children,
      ),
    );
  }
}
