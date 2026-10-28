// Kanso — share_request_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../utils/share_labels.dart';

/// Someone who asked to join through a link: who, the role the link gives
/// and their fingerprint, and the owner's two answers.
class ShareRequestRow extends StatelessWidget {
  const ShareRequestRow({
    required this.request,
    required this.busy,
    required this.onApprove,
    required this.onDecline,
    super.key,
  });

  final MemberRow request;
  final bool busy;
  final VoidCallback onApprove;
  final VoidCallback onDecline;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(
            l10n.shareRequestAsked(request.username),
            style: typography.body,
          ),
          Text(
            l10n.shareRequestDetail(
              l10n.shareRoleName(request.role),
              request.fingerprint,
            ),
            style: typography.meta,
          ),
          const SizedBox(height: KansoSpacing.xxs),
          Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: <Widget>[
              KansoButton(
                label: l10n.shareDecline,
                variant: KansoButtonVariant.quiet,
                expanded: false,
                compact: true,
                onPressed: busy ? null : onDecline,
              ),
              const SizedBox(width: KansoSpacing.xs),
              KansoButton(
                label: l10n.shareApprove,
                variant: KansoButtonVariant.secondary,
                expanded: false,
                compact: true,
                onPressed: busy ? null : onApprove,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
