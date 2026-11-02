// Kanso — close_loop.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../core/bridge/api/types.dart';
import '../../core/models/repos/abs/digest_repository.dart';
import '../../core/utils/context_extensions.dart';
import '../../core/views/widgets/kanso_feedback.dart';
import '../../injection.dart';

/// Closes [item] as [action] and says so when it couldn't: the sentence was
/// gone from its note already, or the write failed. The page reads again
/// by itself when the library changes.
Future<void> closeLoop(
  BuildContext context,
  OpenLoopItem item,
  LoopAction action,
) async {
  final l10n = context.l10n;
  final result = await getIt<DigestRepository>().closeLoop(
    item: item,
    action: action,
  );
  if (!context.mounted) {
    return;
  }
  result.fold(
    (_) => KansoFeedback.show(
      context,
      l10n.digestError,
      tone: KansoFeedbackTone.error,
    ),
    (changed) {
      if (!changed) {
        KansoFeedback.show(
          context,
          l10n.digestLoopGone,
          tone: KansoFeedbackTone.info,
        );
      }
    },
  );
}
