// Kanso — approval_wait_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_code_display.dart';
import '../../state/unlock/unlock_bloc.dart';
import '../../utils/account_labels.dart';
import 'account_pane_frame.dart';

/// Waiting for another device to approve this one: what the wait is doing
/// in plain words, the six digits to compare once a device has answered,
/// the recovery key as another way when it is offered, and Cancel. After
/// the request ended it offers to try again. It needs an [UnlockBloc]
/// above it.
class ApprovalWaitPane extends StatelessWidget {
  const ApprovalWaitPane({
    required this.wait,
    required this.onRecover,
    super.key,
  });

  final ApprovalWait wait;
  final VoidCallback onRecover;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final bloc = context.read<UnlockBloc>();
    final code = wait.code;
    final phase = wait.phase;
    final ended =
        phase is ApprovalPhase_Cancelled ||
        phase is ApprovalPhase_Expired ||
        phase is ApprovalPhase_Failed ||
        phase is ApprovalPhase_NoDevices;
    final offerRecovery =
        wait.offerRecovery || phase is ApprovalPhase_NoDevices;
    return AccountPaneFrame(
      title: l10n.accountApprovalTitle,
      leading: const SizedBox.shrink(),
      children: <Widget>[
        Text(l10n.accountApprovalIntro, style: typography.body),
        const SizedBox(height: KansoSpacing.md),
        Semantics(
          liveRegion: true,
          child: Text(l10n.approvalPhase(phase), style: typography.heading2),
        ),
        if (code != null && !ended) ...<Widget>[
          const SizedBox(height: KansoSpacing.md),
          KansoCodeDisplay(
            text: code,
            semanticLabel: l10n.accountApprovalCode(
              code.replaceAll(' ', '').split('').join(' '),
            ),
            caption: l10n.accountApprovalCompare,
          ),
        ],
        const SizedBox(height: KansoSpacing.lg),
        if (ended)
          KansoButton(
            label: l10n.accountRetry,
            onPressed: () => bloc.add(
              UnlockEvent.approvalRequested(deviceName: deviceNameOf(context)),
            ),
          ),
        if (offerRecovery) ...<Widget>[
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.accountApprovalRecoveryInstead,
            variant: KansoButtonVariant.secondary,
            onPressed: onRecover,
          ),
        ],
        const SizedBox(height: KansoSpacing.xs),
        KansoButton(
          label: l10n.accountCancel,
          variant: KansoButtonVariant.quiet,
          onPressed: () => bloc.add(const UnlockEvent.approvalCancelled()),
        ),
      ],
    );
  }
}
