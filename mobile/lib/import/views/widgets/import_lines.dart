// Kanso: import_lines.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_grouped_section.dart';

/// Sentences from the library in one rounded group, a line each, under an
/// optional [title]. Nothing is drawn for an empty list.
class ImportLines extends StatelessWidget {
  const ImportLines({required this.lines, this.title, super.key});

  final List<String> lines;
  final String? title;

  @override
  Widget build(BuildContext context) {
    if (lines.isEmpty) {
      return const SizedBox.shrink();
    }
    final typography = context.typography;
    return KansoGroupedSection(
      title: title,
      children: <Widget>[
        for (final line in lines)
          Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Align(
              alignment: AlignmentDirectional.centerStart,
              child: Text(line, style: typography.body),
            ),
          ),
      ],
    );
  }
}
