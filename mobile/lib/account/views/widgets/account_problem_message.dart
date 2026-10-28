// Kanso — account_problem_message.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../models/data/form_problem.dart';
import '../../utils/account_labels.dart';

/// Why a form did not go through, in a line above its button: what is wrong
/// with what was typed, or why the call failed. Nothing when neither is so.
class AccountProblemMessage extends StatelessWidget {
  const AccountProblemMessage({this.problem, this.error, super.key});

  final FormProblem? problem;
  final ApplicationError? error;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typed = problem;
    final failed = error;
    final words = typed != null
        ? l10n.formProblem(typed)
        : failed != null
        ? l10n.accountError(failed)
        : null;
    if (words == null) {
      return const SizedBox.shrink();
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      child: KansoInlineMessage(message: words),
    );
  }
}
