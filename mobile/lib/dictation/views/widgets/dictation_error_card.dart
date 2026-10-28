// Kanso — dictation_error_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/dictation_failure.dart';

/// Says that dictation stopped, and why in a few words, with a way to try
/// again.
class DictationErrorCard extends StatelessWidget {
  const DictationErrorCard({
    required this.failure,
    required this.onRetry,
    super.key,
  });

  final DictationFailure failure;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.info,
      title: l10n.dictationErrorTitle,
      message: failure == DictationFailure.noSpeech
          ? l10n.dictationNoSpeech
          : l10n.dictationErrorMessage,
      actions: <Widget>[
        KansoButton(label: l10n.dictationTryAgain, onPressed: onRetry),
      ],
    );
  }
}
