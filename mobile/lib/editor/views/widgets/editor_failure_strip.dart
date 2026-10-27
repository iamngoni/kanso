// Kanso — editor_failure_strip.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// A quiet strip above the text when saving the note failed, with a way to
/// try again. What was typed is safe in the editor.
class EditorFailureStrip extends StatelessWidget {
  const EditorFailureStrip({
    required this.onRetry,
    required this.gutter,
    super.key,
  });

  final VoidCallback onRetry;

  /// The space either side of the strip.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Semantics(
      container: true,
      liveRegion: true,
      label: l10n.editorSaveFailed,
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: gutter),
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.danger.withValues(alpha: 0.12),
            borderRadius: KansoRadii.mediumBorder,
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xxs,
            ),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: ExcludeSemantics(
                    child: Text(
                      l10n.editorSaveFailed,
                      style: context.typography.label,
                    ),
                  ),
                ),
                KansoButton(
                  label: l10n.retry,
                  onPressed: onRetry,
                  variant: KansoButtonVariant.quiet,
                  expanded: false,
                  compact: true,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
