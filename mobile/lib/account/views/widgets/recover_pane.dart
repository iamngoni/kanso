// Kanso — recover_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/form_problem.dart';
import '../../state/recover/recover_bloc.dart';
import 'account_pane_frame.dart';
import 'account_problem_message.dart';

/// Reset your password with the recovery key: the username (filled in and
/// fixed when the phone is locked to an account), the key, and a new
/// password with its repeat. It needs a [RecoverBloc] above it.
class RecoverPane extends StatelessWidget {
  const RecoverPane({required this.state, this.fixedUsername, super.key});

  final RecoverState state;

  /// The account this phone is locked to, when it is.
  final String? fixedUsername;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<RecoverBloc>();
    final fixed = fixedUsername;
    final idle = !state.busy;
    return AccountPaneFrame(
      title: l10n.accountRecoverTitle,
      children: <Widget>[
        Text(l10n.accountRecoverExplain, style: context.typography.body),
        const SizedBox(height: KansoSpacing.md),
        KansoFieldControllers(
          count: 4,
          initialTexts: <String>[fixed ?? ''],
          builder: (context, controllers) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                KansoTextField(
                  controller: controllers[0],
                  label: l10n.accountUsername,
                  textInputAction: TextInputAction.next,
                  autofillHints: const <String>[AutofillHints.username],
                  enabled: idle && fixed == null,
                  invalid: state.problem == FormProblem.usernameInvalid,
                ),
                const SizedBox(height: KansoSpacing.sm),
                KansoTextField(
                  controller: controllers[1],
                  label: l10n.accountRecoveryKeyField,
                  hint: l10n.accountRecoveryKeyHint,
                  mono: true,
                  textInputAction: TextInputAction.next,
                  enabled: idle,
                  invalid: state.problem == FormProblem.recoveryKeyMissing,
                ),
                const SizedBox(height: KansoSpacing.sm),
                KansoTextField(
                  controller: controllers[2],
                  label: l10n.accountNewPassword,
                  obscure: true,
                  textInputAction: TextInputAction.next,
                  autofillHints: const <String>[AutofillHints.newPassword],
                  enabled: idle,
                  invalid:
                      state.problem == FormProblem.passwordTooShort ||
                      state.problem == FormProblem.passwordTooLong ||
                      state.problem == FormProblem.passwordSameAsUsername,
                ),
                const SizedBox(height: KansoSpacing.sm),
                KansoTextField(
                  controller: controllers[3],
                  label: l10n.accountRepeatPassword,
                  obscure: true,
                  textInputAction: TextInputAction.done,
                  enabled: idle,
                  invalid: state.problem == FormProblem.passwordsDiffer,
                ),
                const SizedBox(height: KansoSpacing.md),
                AccountProblemMessage(
                  problem: state.problem,
                  error: state.error,
                ),
                KansoButton(
                  label: state.busy
                      ? l10n.accountRecoveringButton
                      : l10n.accountRecoverButton,
                  onPressed: idle
                      ? () => bloc.add(
                          RecoverEvent.submitted(
                            username: controllers[0].text,
                            recoveryKey: controllers[1].text,
                            password: controllers[2].text,
                            repeat: controllers[3].text,
                            deviceName: deviceNameOf(context),
                          ),
                        )
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
