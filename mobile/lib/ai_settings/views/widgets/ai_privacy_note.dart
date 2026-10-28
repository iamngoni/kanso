// Kanso — ai_privacy_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';

/// The plain line about where a question goes and what is never sent.
class AiPrivacyNote extends StatelessWidget {
  const AiPrivacyNote({required this.text, super.key});

  final String text;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.only(top: 2),
            child: KansoIcon(
              KansoIcons.shield,
              size: KansoSizes.iconSmall,
              color: colors.textMuted,
            ),
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(child: Text(text, style: context.typography.meta)),
        ],
      ),
    );
  }
}
