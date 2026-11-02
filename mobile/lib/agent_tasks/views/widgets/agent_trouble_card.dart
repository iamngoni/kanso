// Kanso — agent_trouble_card.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../utils/agent_trouble.dart';

/// A plain sentence about why an agent task didn't happen, with Try again
/// and AI settings.
class AgentTroubleCard extends StatelessWidget {
  const AgentTroubleCard({
    required this.message,
    required this.onTryAgain,
    required this.onSettings,
    super.key,
  });

  final String message;
  final VoidCallback onTryAgain;
  final VoidCallback onSettings;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoMessageCard(
      icon: KansoIcons.sparkles,
      title: l10n.agentTroubleTitle,
      message: agentTroubleText(l10n, message),
      actions: <Widget>[
        KansoButton(label: l10n.agentTroubleTryAgain, onPressed: onTryAgain),
        KansoButton(
          label: l10n.agentTroubleSettings,
          variant: KansoButtonVariant.secondary,
          onPressed: onSettings,
        ),
      ],
    );
  }
}
