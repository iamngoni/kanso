// Kanso — unlock_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_or_divider.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/form_problem.dart';
import '../../state/unlock/unlock_bloc.dart';
import 'account_pane_frame.dart';
import 'account_problem_message.dart';

/// Open your notes on this device: a line on why they are closed, the
/// password when the account has one, and the other ways in: approve from
/// another device, the recovery key, or signing out. It needs an
/// [UnlockBloc] above it; [onRecover] opens the recovery form.
class UnlockPane extends StatelessWidget {
  const UnlockPane({
    required this.state,
    required this.info,
    required this.onRecover,
    super.key,
  });

  final UnlockState state;
  final LockedInfo info;
  final VoidCallback onRecover;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<UnlockBloc>();
    final idle = !state.busy;
    return AccountPaneFrame(
      title: l10n.accountUnlockTitle,
      children: <Widget>[
        Text(
          l10n.accountUnlockExplain(info.username),
          style: context.typography.body,
        ),
        const SizedBox(height: KansoSpacing.lg),
        KansoFieldControllers(
          count: 1,
          builder: (context, controllers) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                if (info.hasPassword) ...<Widget>[
                  KansoTextField(
                    controller: controllers[0],
                    label: l10n.accountPassword,
                    obscure: true,
                    textInputAction: TextInputAction.done,
                    autofillHints: const <String>[AutofillHints.password],
                    enabled: idle,
                    invalid: state.problem == FormProblem.credentialsDontMatch,
                    onSubmitted: (password) => bloc.add(
                      UnlockEvent.passwordSubmitted(
                        username: info.username,
                        password: password,
                      ),
                    ),
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  AccountProblemMessage(
                    problem: state.problem,
                    error: state.error,
                  ),
                  KansoButton(
                    label: l10n.accountUnlockButton,
                    onPressed: idle
                        ? () => bloc.add(
                            UnlockEvent.passwordSubmitted(
                              username: info.username,
                              password: controllers[0].text,
                            ),
                          )
                        : null,
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  KansoOrDivider(label: l10n.accountOr),
                  const SizedBox(height: KansoSpacing.md),
                ] else
                  AccountProblemMessage(
                    problem: state.problem,
                    error: state.error,
                  ),
                KansoButton(
                  label: l10n.accountApproveFromDevice,
                  variant: KansoButtonVariant.secondary,
                  onPressed: idle
                      ? () => bloc.add(
                          UnlockEvent.approvalRequested(
                            deviceName: deviceNameOf(context),
                          ),
                        )
                      : null,
                ),
                const SizedBox(height: KansoSpacing.xs),
                KansoButton(
                  label: l10n.accountUseRecoveryKey,
                  variant: KansoButtonVariant.secondary,
                  onPressed: idle ? onRecover : null,
                ),
                const SizedBox(height: KansoSpacing.md),
                KansoButton(
                  label: l10n.accountSignOut,
                  variant: KansoButtonVariant.quiet,
                  onPressed: idle
                      ? () => bloc.add(const UnlockEvent.signedOut())
                      : null,
                ),
              ],
            );
          },
        ),
      ],
    );
  }
}
