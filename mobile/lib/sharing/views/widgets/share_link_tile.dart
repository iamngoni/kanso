// Kanso — share_link_tile.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../utils/days_until.dart';
import '../../utils/share_labels.dart';

/// An invite link of the note: its role and how long it still works, and
/// the owner's way to cancel it. The link's address is not here: it is
/// known only to the call that made it.
class ShareLinkTile extends StatelessWidget {
  const ShareLinkTile({
    required this.link,
    required this.busy,
    required this.onCancel,
    super.key,
  });

  final ShareLinkRow link;
  final bool busy;
  final VoidCallback onCancel;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final days = daysUntil(link.expiresMs, clock.now().millisecondsSinceEpoch);
    return Row(
      children: <Widget>[
        KansoIcon(
          KansoIcons.link,
          size: 16,
          color: context.colors.textSecondary,
        ),
        const SizedBox(width: KansoSpacing.sm),
        Expanded(
          child: Text(
            l10n.shareLinkText(link.role, days),
            style: context.typography.body,
          ),
        ),
        KansoButton(
          label: l10n.shareLinkCancel,
          variant: KansoButtonVariant.quiet,
          expanded: false,
          compact: true,
          onPressed: busy ? null : onCancel,
        ),
      ],
    );
  }
}
