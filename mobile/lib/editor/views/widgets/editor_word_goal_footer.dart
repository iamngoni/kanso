// Kanso — editor_word_goal_footer.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/word_goal.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// The live word count and note target, pinned below the editor text.
class EditorWordGoalFooter extends StatelessWidget {
  const EditorWordGoalFooter({
    required this.words,
    required this.goal,
    required this.onTap,
    super.key,
  });

  final int words;
  final int? goal;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final target = goal;
    final label = target == null
        ? context.l10n.editorWordGoalSet
        : context.l10n.editorWordGoalProgress(
            wordGoalFormatCount(count: words),
            wordGoalFormatCount(count: target),
          );
    final progress = target == null
        ? 0.0
        : wordGoalProgress(words: words, goal: target);
    final met = target != null && progress >= 1;
    final ink = met ? colors.success : colors.textSecondary;
    return Semantics(
      button: onTap != null,
      label: label,
      child: InkWell(
        onTap: onTap,
        child: Container(
          height: 44,
          decoration: BoxDecoration(
            border: Border(top: BorderSide(color: colors.divider)),
          ),
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: <Widget>[
              if (target != null) ...<Widget>[
                SizedBox.square(
                  dimension: 14,
                  child: CircularProgressIndicator(
                    value: progress,
                    strokeWidth: 1.75,
                    backgroundColor: colors.divider,
                    color: met ? colors.success : colors.accent,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
              ],
              Text(
                target == null
                    ? label
                    : wordGoalLabel(words: words, goal: target),
                style: context.typography.meta.copyWith(color: ink),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
