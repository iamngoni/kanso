// Kanso — fingerprint_block.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// A fingerprint to compare out loud, in its groups of digits, in a
/// monospaced face on a quiet box. [label] names it to a screen reader.
class FingerprintBlock extends StatelessWidget {
  const FingerprintBlock({
    required this.fingerprint,
    required this.label,
    super.key,
  });

  final String fingerprint;
  final String label;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final groups = fingerprint
        .split(RegExp(r'\s+'))
        .where((group) => group.isNotEmpty)
        .toList();
    return Semantics(
      label: label,
      value: groups.join(', '),
      child: ExcludeSemantics(
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.all(KansoSpacing.sm),
          decoration: BoxDecoration(
            color: colors.list,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(color: colors.divider),
          ),
          child: Wrap(
            spacing: KansoSpacing.sm,
            runSpacing: KansoSpacing.xxs,
            children: <Widget>[
              for (final group in groups)
                Text(group, style: context.typography.mono),
            ],
          ),
        ),
      ),
    );
  }
}
