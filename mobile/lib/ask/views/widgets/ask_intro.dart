// Kanso — ask_intro.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon.dart';

/// What an empty conversation says: what Ask does and, once, where a
/// question goes and what is never sent.
class AskIntro extends StatelessWidget {
  const AskIntro({required this.privacy, super.key});

  /// The core's line about where a question goes; none is drawn when empty.
  final String privacy;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: <Widget>[
        Flexible(
          child: KansoEmptyState(
            title: l10n.askTitle,
            message: l10n.askEmptyMessage,
          ),
        ),
        if (privacy.isNotEmpty)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.lg,
              0,
              KansoSpacing.lg,
              KansoSpacing.lg,
            ),
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
                Expanded(child: Text(privacy, style: context.typography.meta)),
              ],
            ),
          ),
      ],
    );
  }
}
