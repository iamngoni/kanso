// Kanso — share_message_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/share_done.dart';
import '../../models/data/share_problem.dart';
import '../../utils/share_labels.dart';

/// What the last step of the share sheet did, or why it did not happen: a
/// plain line, the danger colour for a problem.
class ShareMessageText extends StatelessWidget {
  const ShareMessageText({this.done, this.problem, super.key});

  final ShareDone? done;
  final ShareProblem? problem;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final failed = problem;
    final finished = done;
    if (failed == null && finished == null) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    return Semantics(
      liveRegion: true,
      child: Text(
        failed != null
            ? l10n.shareProblemText(failed)
            : l10n.shareDoneText(finished!),
        style: context.typography.secondary.copyWith(
          color: failed != null ? colors.danger : colors.textSecondary,
        ),
      ),
    );
  }
}
