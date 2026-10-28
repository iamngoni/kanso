// Kanso — dictation_denied_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';

/// Says that Kanso may not listen, and takes the person to Settings to allow
/// it; where the phone's rules bar it, there is nothing to open.
class DictationDeniedCard extends StatelessWidget {
  const DictationDeniedCard({
    required this.restricted,
    required this.onOpenSettings,
    super.key,
  });

  /// Whether the phone's rules, not the person, bar it.
  final bool restricted;
  final VoidCallback onOpenSettings;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.micOff,
      title: l10n.dictationDeniedTitle,
      message: restricted
          ? l10n.dictationRestrictedMessage
          : l10n.dictationDeniedMessage,
      actions: <Widget>[
        if (!restricted)
          KansoButton(
            label: l10n.dictationOpenSettings,
            onPressed: onOpenSettings,
          ),
      ],
    );
  }
}
