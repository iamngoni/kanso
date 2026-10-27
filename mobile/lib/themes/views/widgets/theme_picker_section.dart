// Kanso — theme_picker_section.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_section_header.dart';

/// A titled row of theme cards: the heading, one sentence on what the cards
/// colour, and the [child] row.
class ThemePickerSection extends StatelessWidget {
  const ThemePickerSection({
    required this.title,
    required this.detail,
    required this.child,
    super.key,
  });

  final String title;
  final String detail;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        KansoSectionHeader(label: title),
        Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            0,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(detail, style: context.typography.meta),
        ),
        child,
      ],
    );
  }
}
