// Kanso — daily_note_failed_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// What shows when today's note could not be opened: what happened, and
/// buttons to try again or to go to the notes instead.
class DailyNoteFailedPane extends StatelessWidget {
  const DailyNoteFailedPane({
    required this.onRetry,
    required this.onLeave,
    super.key,
  });

  /// Tries again.
  final VoidCallback onRetry;

  /// Goes to All Notes.
  final VoidCallback onLeave;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: KansoMessageCard(
          icon: KansoIcons.calendar,
          title: l10n.dailyNoteFailedTitle,
          message: l10n.dailyNoteFailedMessage,
          actions: <Widget>[
            KansoButton(label: l10n.retry, onPressed: onRetry),
            KansoButton(
              label: l10n.libraryAllNotes,
              variant: KansoButtonVariant.quiet,
              onPressed: onLeave,
            ),
          ],
        ),
      ),
    );
  }
}
