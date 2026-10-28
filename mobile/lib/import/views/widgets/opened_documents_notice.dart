// Kanso: opened_documents_notice.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// What could not be done with the documents other apps opened in Kanso: a
/// sentence each from the library, or one for a failure as a whole.
class OpenedDocumentsNotice extends StatelessWidget {
  const OpenedDocumentsNotice({
    required this.problems,
    required this.failed,
    required this.onDismiss,
    super.key,
  });

  final List<String> problems;
  final bool failed;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.circleX,
      title: failed
          ? l10n.openedDocumentsFailedTitle
          : l10n.openedDocumentsProblemsTitle,
      message: failed ? l10n.openedDocumentsFailedMessage : problems.join('\n'),
      actions: <Widget>[
        KansoButton(label: l10n.openedDocumentsOk, onPressed: onDismiss),
      ],
    );
  }
}
