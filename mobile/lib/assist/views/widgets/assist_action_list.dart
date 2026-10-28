// Kanso — assist_action_list.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/assist_action.dart';
import '../../../core/models/data/assist_action_info.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';

/// What the assistant can do with the text: the actions in the core's words,
/// those that need text selected left out while none is, and, for the note
/// as a whole, its summary.
class AssistActionList extends StatelessWidget {
  const AssistActionList({
    required this.actions,
    required this.hasSelection,
    required this.hasText,
    required this.onChoose,
    required this.onSummarizeNote,
    super.key,
  });

  final List<AssistActionInfo> actions;
  final bool hasSelection;

  /// Whether there is any text to work on, selected or before the caret.
  final bool hasText;
  final ValueChanged<AssistAction> onChoose;
  final VoidCallback onSummarizeNote;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final offered = <AssistActionInfo>[
      for (final info in actions)
        if (hasText && (hasSelection || !info.needsSelection)) info,
    ];
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Text(l10n.assistTitle, style: context.typography.heading2),
        ),
        for (final info in offered) ...<Widget>[
          const KansoDivider(indent: KansoSpacing.md),
          KansoSettingsRow(
            icon: KansoIcons.sparkles,
            label: info.label,
            onTap: () => onChoose(info.action),
          ),
        ],
        if (!hasSelection) ...<Widget>[
          const KansoDivider(indent: KansoSpacing.md),
          KansoSettingsRow(
            icon: KansoIcons.fileText,
            label: l10n.assistSummarizeNote,
            onTap: onSummarizeNote,
          ),
        ],
        const SizedBox(height: KansoSpacing.md),
      ],
    );
  }
}
