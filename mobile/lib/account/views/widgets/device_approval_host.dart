// Kanso — device_approval_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/account/account_bloc.dart';
import '../../../core/state/device_approval/device_approval_bloc.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_plain_text.dart';
import '../../../core/views/widgets/kanso_scrim.dart';
import 'approve_device_sheet.dart';

/// [child] with the sheet that asks to let a device in over it, on a veil,
/// while a request waits. Nothing below it can be touched until it is
/// answered. It needs the app's `DeviceApprovalBloc` and `AccountBloc`.
class DeviceApprovalHost extends StatelessWidget {
  const DeviceApprovalHost({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DeviceApprovalBloc, DeviceApprovalState>(
      builder: (context, state) {
        final prompt = state.shown;
        return Stack(
          children: <Widget>[
            Positioned.fill(child: child),
            if (prompt != null) ...<Widget>[
              const Positioned.fill(child: AbsorbPointer(child: KansoScrim())),
              Positioned.fill(
                // Above the app's navigator there is no page to give the
                // text its style, so the sheet sets plain text itself.
                child: KansoPlainText(
                  child: Align(
                    alignment: Alignment.bottomCenter,
                    child: BlocBuilder<AccountBloc, AccountBlocState>(
                      builder: (context, account) {
                        final bloc = context.read<DeviceApprovalBloc>();
                        return ApproveDeviceSheet(
                          prompt: prompt,
                          account: account.username ?? '',
                          waiting: state.prompts.length - 1,
                          busy: state.answering != null,
                          error: state.error == null
                              ? null
                              : context.l10n.accountError(state.error!),
                          onApprove: () =>
                              bloc.add(DeviceApprovalEvent.approved(prompt.id)),
                          onDecline: () =>
                              bloc.add(DeviceApprovalEvent.declined(prompt.id)),
                        );
                      },
                    ),
                  ),
                ),
              ),
            ],
          ],
        );
      },
    );
  }
}
