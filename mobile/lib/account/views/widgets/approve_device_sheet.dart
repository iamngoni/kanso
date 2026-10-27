// Kanso — approve_device_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_code_display.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';

/// The sheet that asks this phone to let another device into the account:
/// who asks to join and when, the six digits to compare with the other
/// device's screen, Approve and Don't approve. A request that changed after
/// it was made is declined only: it says so and offers just Don't approve.
class ApproveDeviceSheet extends StatelessWidget {
  const ApproveDeviceSheet({
    required this.prompt,
    required this.account,
    required this.onApprove,
    required this.onDecline,
    this.waiting = 0,
    this.busy = false,
    this.error,
    super.key,
  });

  final ApprovalPrompt prompt;

  /// The name of the account the device asks to join.
  final String account;

  /// Approves the request; not shown for one that changed.
  final VoidCallback onApprove;
  final VoidCallback onDecline;

  /// How many more requests wait behind this one.
  final int waiting;

  /// Whether the answer is being sent.
  final bool busy;

  /// Why the last answer failed.
  final String? error;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final locale = Localizations.localeOf(context).toString();
    final time = DateFormat.Hm(
      locale,
    ).format(DateTime.fromMillisecondsSinceEpoch(prompt.askedAtMs));
    final code = prompt.code;
    final changed = prompt.state == PromptState.changed;
    return KansoBottomSheet(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            Text(l10n.approveDeviceTitle, style: typography.heading2),
            const SizedBox(height: KansoSpacing.xs),
            Text(
              l10n.approveDeviceMessage(
                account,
                prompt.deviceName ?? l10n.approveDeviceUnnamed,
                time,
              ),
              style: typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.md),
            if (changed)
              KansoInlineMessage(message: l10n.approveDeviceChanged)
            else if (code != null) ...<Widget>[
              KansoCodeDisplay(
                text: code,
                semanticLabel: l10n.accountApprovalCode(
                  code.replaceAll(' ', '').split('').join(' '),
                ),
              ),
              const SizedBox(height: KansoSpacing.xs),
              Text(l10n.approveDeviceCompare(code), style: typography.body),
            ],
            if (error != null) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              KansoInlineMessage(message: error!),
            ],
            const SizedBox(height: KansoSpacing.md),
            if (!changed) ...<Widget>[
              KansoButton(
                label: l10n.approveDeviceApprove,
                onPressed: busy ? null : onApprove,
              ),
              const SizedBox(height: KansoSpacing.xs),
            ],
            KansoButton(
              label: l10n.approveDeviceDecline,
              variant: changed
                  ? KansoButtonVariant.primary
                  : KansoButtonVariant.secondary,
              onPressed: busy ? null : onDecline,
            ),
            if (waiting > 0) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              Text(
                l10n.approveDeviceMore(waiting),
                textAlign: TextAlign.center,
                style: typography.meta,
              ),
            ],
          ],
        ),
      ),
    );
  }
}
