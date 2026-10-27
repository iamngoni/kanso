// Kanso — editor_conflict_strip.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// The strip above the text when the note was changed (or deleted) elsewhere
/// while it was being edited: what happened, and the two ways to answer, in
/// the desktop's words. The version that is not kept stays in the note's
/// history.
///
/// It is a live region, so a screen reader says it when it appears.
class EditorConflictStrip extends StatelessWidget {
  const EditorConflictStrip({
    required this.deleted,
    required this.onKeepMine,
    required this.onUseTheirs,
    required this.gutter,
    super.key,
  });

  /// Whether the note was deleted rather than changed.
  final bool deleted;

  /// Keep what was typed here; for a deleted note, keep the note.
  final VoidCallback onKeepMine;

  /// Take the other version; for a deleted note, let the note go.
  final VoidCallback onUseTheirs;

  /// The space either side of the strip.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final message = deleted
        ? l10n.editorDeletedMessage
        : l10n.editorChangedMessage;
    return Semantics(
      container: true,
      liveRegion: true,
      label: message,
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: gutter),
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.warning.withValues(alpha: 0.16),
            borderRadius: KansoRadii.mediumBorder,
          ),
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                ExcludeSemantics(
                  child: Text(message, style: context.typography.label),
                ),
                const SizedBox(height: KansoSpacing.xs),
                Row(
                  children: <Widget>[
                    KansoButton(
                      label: deleted ? l10n.editorKeepIt : l10n.editorKeepMine,
                      onPressed: onKeepMine,
                      expanded: false,
                      compact: true,
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    KansoButton(
                      label: deleted
                          ? l10n.editorDiscard
                          : l10n.editorUseTheirs,
                      onPressed: onUseTheirs,
                      variant: KansoButtonVariant.secondary,
                      expanded: false,
                      compact: true,
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
