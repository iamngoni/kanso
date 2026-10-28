// Kanso — ai_field_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_text_field.dart';

/// A labelled line of text in a settings section: the label above, the field
/// under it showing [hint] (the provider's own default) while empty.
class AiFieldRow extends StatelessWidget {
  const AiFieldRow({
    required this.label,
    required this.hint,
    required this.controller,
    required this.onChanged,
    this.keyboardType = TextInputType.text,
    super.key,
  });

  final String label;
  final String hint;
  final TextEditingController controller;
  final ValueChanged<String> onChanged;
  final TextInputType keyboardType;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.sm,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(
            label,
            style: context.typography.label.copyWith(
              color: colors.textSecondary,
            ),
          ),
          const SizedBox(height: KansoSpacing.xxs),
          KansoTextField(
            controller: controller,
            hint: hint,
            semanticLabel: label,
            keyboardType: keyboardType,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}
