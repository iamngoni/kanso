// Kanso — invitation_tile.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../utils/share_labels.dart';

/// A note someone shared with this account: who it is from and what they
/// can do with it, and the two answers. The note's own title is sealed until
/// it is accepted, so the invitation names only the person.
class InvitationTile extends StatelessWidget {
  const InvitationTile({
    required this.invitation,
    required this.working,
    required this.onAccept,
    required this.onDecline,
    super.key,
  });

  final InvitationRow invitation;

  /// The answer is being sent: nothing can be pressed.
  final bool working;
  final VoidCallback onAccept;
  final VoidCallback onDecline;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.all(KansoSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              KansoIcon(KansoIcons.users, color: context.colors.textSecondary),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(
                      l10n.sharedInvitationFrom(invitation.from),
                      style: typography.listTitle,
                    ),
                    Text(
                      l10n.shareRoleName(invitation.role),
                      style: typography.meta,
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.sm),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: <Widget>[
              KansoButton(
                label: l10n.shareDecline,
                variant: KansoButtonVariant.quiet,
                expanded: false,
                compact: true,
                onPressed: working ? null : onDecline,
              ),
              const SizedBox(width: KansoSpacing.xs),
              KansoButton(
                label: l10n.sharedAccept,
                expanded: false,
                compact: true,
                onPressed: working ? null : onAccept,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
