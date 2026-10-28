// Kanso — sign_in_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_or_divider.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
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

/// Sign in: the server, the ways in that take no typing that the server
/// offers (Apple, Google, GitHub, a passkey), and a username and password,
/// with the ways to make an account and to get in with the recovery key.
///
/// It needs a [SignInBloc] above it; [onRecover] opens the recovery form for
/// the server in use.
class SignInPane extends StatelessWidget {
  const SignInPane({required this.state, required this.onRecover, super.key});

  final SignInState state;
  final ValueChanged<String> onRecover;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    final methods = state.methods;
    return AccountPaneFrame(
      title: l10n.accountSignInTitle,
      children: <Widget>[
        Text(l10n.accountSignInHeading, style: context.typography.largeTitle),
        const SizedBox(height: KansoSpacing.md),
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
            count: 2,
            builder: (context, controllers) {
              final idle = state.busy == null;
              final signIn = methods.password
                  ? <Widget>[
                      KansoTextField(
                        controller: controllers[0],
                        label: l10n.accountUsername,
                        textInputAction: TextInputAction.next,
                        autofillHints: const <String>[AutofillHints.username],
                        enabled: idle,
                        invalid:
                            state.problem == FormProblem.usernameInvalid ||
                            state.problem == FormProblem.credentialsDontMatch,
                      ),
                      const SizedBox(height: KansoSpacing.sm),
                      KansoTextField(
                        controller: controllers[1],
                        label: l10n.accountPassword,
                        obscure: true,
                        textInputAction: TextInputAction.done,
                        autofillHints: const <String>[AutofillHints.password],
                        enabled: idle,
                        invalid:
                            state.problem == FormProblem.credentialsDontMatch,
                      ),
                      const SizedBox(height: KansoSpacing.md),
                      AccountProblemMessage(
                        problem: state.problem,
                        error: state.error,
                      ),
                      KansoButton(
                        label: state.busy == SignInAction.password
                            ? l10n.accountSigningIn
                            : l10n.accountSignInButton,
                        onPressed: idle
                            ? () => bloc.add(
                                SignInEvent.passwordSubmitted(
                                  username: controllers[0].text,
                                  password: controllers[1].text,
                                  deviceName: deviceNameOf(context),
                                ),
                              )
                            : null,
                      ),
                      const SizedBox(height: KansoSpacing.xs),
                      KansoPressable(
                        onTap: idle ? () => onRecover(state.server) : null,
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            vertical: KansoSpacing.sm,
                          ),
                          child: Text(
                            l10n.accountForgotPassword,
                            textAlign: TextAlign.center,
                            style: context.typography.label,
                          ),
                        ),
                      ),
                    ]
                  : <Widget>[
                      AccountProblemMessage(
                        problem: state.problem,
                        error: state.error,
                      ),
                    ];
              return Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  if (SignInMethods.shows(methods)) ...<Widget>[
                    SignInMethods(state: state, methods: methods),
                    if (methods.password) ...<Widget>[
                      const SizedBox(height: KansoSpacing.md),
                      KansoOrDivider(label: l10n.accountOr),
                      const SizedBox(height: KansoSpacing.md),
                    ] else if (state.error != null)
                      const SizedBox(height: KansoSpacing.md),
                  ],
                  if (!methods.password && !SignInMethods.shows(methods))
                    KansoInlineMessage(
                      message: l10n.accountServerNoMethods,
                      tone: KansoMessageTone.info,
                    ),
                  ...signIn,
                  if (methods.password || methods.passkeys) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xs),
                    KansoButton(
                      label: l10n.accountCreateAccountLink,
                      variant: KansoButtonVariant.quiet,
                      onPressed: idle
                          ? () => bloc.add(
                              const SignInEvent.modeChanged(SignInMode.create),
                            )
                          : null,
                    ),
                  ],
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
