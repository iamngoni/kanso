// Kanso — ai_test_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/ai_test_status.dart';

/// The Test button and what it found: OK with what the model said, or why it
/// failed in the words the core gave.
class AiTestRow extends StatelessWidget {
  const AiTestRow({
    required this.status,
    required this.message,
    required this.onTest,
    super.key,
  });

  final AiTestStatus status;

  /// What the model said, or why the test failed.
  final String message;
  final VoidCallback onTest;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final (glyph, ink, line) = switch (status) {
      AiTestStatus.idle => (null, colors.textMuted, l10n.aiTestHint),
      AiTestStatus.running => (null, colors.textSecondary, l10n.aiTestRunning),
      AiTestStatus.ok => (
        KansoIcons.circleCheck,
        colors.success,
        l10n.aiTestOk(message),
      ),
      AiTestStatus.failed => (KansoIcons.circleX, colors.danger, message),
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        KansoButton(
          label: l10n.aiTest,
          variant: KansoButtonVariant.secondary,
          onPressed: status == AiTestStatus.running ? null : onTest,
        ),
        const SizedBox(height: KansoSpacing.xs),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              if (glyph != null) ...<Widget>[
                KansoIcon(glyph, size: KansoSizes.iconSmall, color: ink),
                const SizedBox(width: KansoSpacing.xs),
              ],
              Expanded(
                child: Semantics(
                  liveRegion: status != AiTestStatus.idle,
                  child: Text(
                    line,
                    style: typography.meta.copyWith(color: ink),
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
