// Kanso — receipt_mark.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';

/// A receipt in an answer: a small raised number after the sentence it backs.
/// A tap shows the lines it points at; a long press opens the note there.
class ReceiptMark extends StatelessWidget {
  const ReceiptMark({
    required this.label,
    required this.semanticLabel,
    required this.onTap,
    required this.onLongPress,
    super.key,
  });

  /// The number as superscript digits.
  final String label;
  final String semanticLabel;
  final VoidCallback onTap;
  final VoidCallback onLongPress;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      button: true,
      label: semanticLabel,
      excludeSemantics: true,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        onLongPress: onLongPress,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 2, vertical: 6),
          child: Text(
            label,
            style: context.typography.body.copyWith(
              color: colors.accent,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ),
    );
  }
}
