// Kanso — voice_note_save_failed_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// Says that the voice note could not be saved and that the words are still
/// there, with a way to try again.
class VoiceNoteSaveFailedCard extends StatelessWidget {
  const VoiceNoteSaveFailedCard({required this.onRetry, super.key});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.info,
      title: l10n.dictationSaveFailedTitle,
      message: l10n.dictationSaveFailedMessage,
      actions: <Widget>[
        KansoButton(label: l10n.dictationTryAgain, onPressed: onRetry),
      ],
    );
  }
}
