// Kanso — dictation_unavailable_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/dictation_unavailable_reason.dart';

/// Says that this phone cannot turn speech into text, and why, in words; the
/// one thing to do is leave.
class DictationUnavailableCard extends StatelessWidget {
  const DictationUnavailableCard({
    required this.reason,
    required this.onClose,
    super.key,
  });

  final DictationUnavailableReason reason;
  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.micOff,
      title: l10n.dictationUnavailableTitle,
      message: switch (reason) {
        DictationUnavailableReason.noRecognizer =>
          l10n.dictationUnavailableNoRecognizer,
        DictationUnavailableReason.notOnDevice =>
          l10n.dictationUnavailableNotOnDevice,
        DictationUnavailableReason.unsupportedLanguage =>
          l10n.dictationUnavailableLanguage,
      },
      actions: <Widget>[
        KansoButton(
          label: l10n.dictationCancel,
          variant: KansoButtonVariant.secondary,
          onPressed: onClose,
        ),
      ],
    );
  }
}
