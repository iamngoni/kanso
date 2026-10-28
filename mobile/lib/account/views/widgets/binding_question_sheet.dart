// Kanso — binding_question_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// The question a sign-in asks when this phone has notes that are not the
/// account's: its title and what it found, and a button for each answer the
/// library offers (Add, and Replace unless the account is new), with what
/// each does, and Cancel.
///
/// [show] completes with the answer, or with null for Cancel and for turning
/// the sheet away.
class BindingQuestionSheet extends StatelessWidget {
  const BindingQuestionSheet({required this.question, super.key});

  final BindingQuestion question;

  /// Asks [question] in a sheet above the app.
  static Future<BindingAnswer?> show(
    BuildContext context,
    BindingQuestion question,
  ) {
    return showKansoBottomSheet<BindingAnswer>(
      context,
      builder: (context) => BindingQuestionSheet(question: question),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return SingleChildScrollView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          Text(question.title, style: typography.heading2),
          const SizedBox(height: KansoSpacing.xs),
          Text(question.body, style: typography.secondary),
          for (final answer in question.answers) ...<Widget>[
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: switch (answer) {
                BindingAnswer.add => l10n.bindingAdd,
                BindingAnswer.replace => l10n.bindingReplace,
              },
              variant: switch (answer) {
                BindingAnswer.add => KansoButtonVariant.primary,
                BindingAnswer.replace => KansoButtonVariant.secondary,
              },
              onPressed: () => Navigator.of(context).pop(answer),
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Text(switch (answer) {
              BindingAnswer.add => l10n.bindingAddNote,
              BindingAnswer.replace => l10n.bindingReplaceNote,
            }, style: typography.meta),
          ],
          const SizedBox(height: KansoSpacing.md),
          KansoButton(
            label: l10n.bindingCancel,
            variant: KansoButtonVariant.quiet,
            onPressed: () => Navigator.of(context).pop(),
          ),
        ],
      ),
    );
  }
}
