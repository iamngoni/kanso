// Kanso — create_account_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_or_divider.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/form_problem.dart';
import '../../models/data/sign_in_action.dart';
import '../../models/data/sign_in_mode.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import 'account_pane_frame.dart';
import 'account_problem_message.dart';
import 'account_server_row.dart';
import 'sign_in_methods.dart';
import 'sign_in_methods_loading.dart';

/// Create your account: a username, a password and its repeat, or a passkey
/// when the server offers them, with a line that a recovery key comes next.
/// It needs a [SignInBloc] above it.
class CreateAccountPane extends StatelessWidget {
  const CreateAccountPane({required this.state, super.key});

  final SignInState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    final methods = state.methods;
    return AccountPaneFrame(
      title: l10n.accountCreateTitle,
      leading: KansoIconButton(
        icon: KansoIcons.chevronLeft,
        semanticLabel: l10n.settingsBack,
        onPressed: state.busy == null
            ? () => bloc.add(const SignInEvent.modeChanged(SignInMode.signIn))
            : null,
      ),
      children: <Widget>[
        AccountServerRow(state: state),
        const SizedBox(height: KansoSpacing.lg),
        switch (state.methodsStatus) {
          LoadStatus.loading => const SignInMethodsLoading(),
          LoadStatus.failed => Center(
            child: KansoMessageCard(
              title: l10n.accountServerFailedTitle,
              message: state.error == null
                  ? l10n.accountSomethingWrong
                  : l10n.accountError(state.error!),
              actions: <Widget>[
                KansoButton(
                  label: l10n.accountRetry,
                  onPressed: () => bloc.add(const SignInEvent.methodsRetried()),
                ),
              ],
            ),
          ),
          LoadStatus.ready when methods != null => KansoFieldControllers(
            count: 3,
            builder: (context, controllers) {
              final idle = state.busy == null;
              final open = methods.signupsOpen;
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  if (!open) ...<Widget>[
                    KansoInlineMessage(
                      message: l10n.accountServerNoSignups,
                      tone: KansoMessageTone.info,
                    ),
                    const SizedBox(height: KansoSpacing.md),
                  ],
                  KansoTextField(
                    controller: controllers[0],
                    label: l10n.accountUsername,
                    textInputAction: TextInputAction.next,
                    autofillHints: const <String>[AutofillHints.newUsername],
                    enabled: idle,
                    invalid:
                        state.problem == FormProblem.usernameInvalid ||
                        state.problem == FormProblem.usernameReserved,
                  ),
                  if (methods.password) ...<Widget>[
                    const SizedBox(height: KansoSpacing.sm),
                    KansoTextField(
                      controller: controllers[1],
                      label: l10n.accountPassword,
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
                      controller: controllers[2],
                      label: l10n.accountRepeatPassword,
                      obscure: true,
                      textInputAction: TextInputAction.done,
                      enabled: idle,
                      invalid: state.problem == FormProblem.passwordsDiffer,
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.md),
                  Text(
                    l10n.accountCreateRecoveryNote,
                    style: context.typography.secondary,
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  AccountProblemMessage(
                    problem: state.problem,
                    error: state.error,
                  ),
                  if (methods.password)
                    KansoButton(
                      label: state.busy == SignInAction.create
                          ? l10n.accountCreatingButton
                          : l10n.accountCreateButton,
                      onPressed: idle && open
                          ? () => bloc.add(
                              SignInEvent.signUpSubmitted(
                                username: controllers[0].text,
                                password: controllers[1].text,
                                repeat: controllers[2].text,
                                deviceName: deviceNameOf(context),
                              ),
                            )
                          : null,
                    ),
                  if (methods.passkeys) ...<Widget>[
                    if (methods.password) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoOrDivider(label: l10n.accountOr),
                      const SizedBox(height: KansoSpacing.md),
                    ],
                    SignInMethods(
                      state: state,
                      methods: methods,
                      creating: true,
                      enabled: open,
                      passkeyUsername: () => controllers[0].text,
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.xs),
                  KansoButton(
                    label: l10n.accountSignInInstead,
                    variant: KansoButtonVariant.quiet,
                    onPressed: idle
                        ? () => bloc.add(
                            const SignInEvent.modeChanged(SignInMode.signIn),
                          )
                        : null,
                  ),
                ],
              );
            },
          ),
          LoadStatus.ready => const SizedBox.shrink(),
        },
      ],
    );
  }
}
