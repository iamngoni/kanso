// Kanso — kanso_or_divider.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// A line with a word in the middle, between two ways of doing one thing:
/// "or".
class KansoOrDivider extends StatelessWidget {
  const KansoOrDivider({required this.label, super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: <Widget>[
        const Expanded(child: KansoDivider()),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
          child: Text(label, style: context.typography.meta),
        ),
        const Expanded(child: KansoDivider()),
      ],
    );
  }
}
