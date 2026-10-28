// Kanso — choose_username_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_field_controllers.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../models/data/form_problem.dart';
import '../../models/data/sign_in_action.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import 'account_pane_frame.dart';
import 'account_problem_message.dart';

/// Choose a username: the name the library suggests for the identity a
/// provider vouched for, to keep or change, and Create account. It needs a
/// [SignInBloc] above it.
class ChooseUsernamePane extends StatelessWidget {
  const ChooseUsernamePane({
    required this.state,
    required this.suggested,
    super.key,
  });

  final SignInState state;
  final String suggested;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    final idle = state.busy == null;
    return AccountPaneFrame(
      title: l10n.accountUsernameTitle,
      leading: const SizedBox.shrink(),
      children: <Widget>[
        Text(l10n.accountUsernameExplain, style: context.typography.body),
        const SizedBox(height: KansoSpacing.md),
        KansoFieldControllers(
          count: 1,
          initialTexts: <String>[suggested],
          builder: (context, controllers) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                KansoTextField(
                  controller: controllers[0],
                  label: l10n.accountUsername,
                  textInputAction: TextInputAction.done,
                  autofillHints: const <String>[AutofillHints.newUsername],
                  autofocus: true,
                  enabled: idle,
                  invalid:
                      state.problem == FormProblem.usernameInvalid ||
                      state.problem == FormProblem.usernameReserved,
                  onSubmitted: (name) =>
                      bloc.add(SignInEvent.usernameChosen(name)),
                ),
                const SizedBox(height: KansoSpacing.md),
                AccountProblemMessage(
                  problem: state.problem,
                  error: state.error,
                ),
                KansoButton(
                  label: state.busy == SignInAction.username
                      ? l10n.accountCreatingButton
                      : l10n.accountCreateButton,
                  onPressed: idle
                      ? () => bloc.add(
                          SignInEvent.usernameChosen(controllers[0].text),
                        )
                      : null,
                ),
                const SizedBox(height: KansoSpacing.xs),
                KansoButton(
                  label: l10n.accountCancel,
                  variant: KansoButtonVariant.quiet,
                  onPressed: idle
                      ? () => bloc.add(const SignInEvent.providerCancelled())
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
