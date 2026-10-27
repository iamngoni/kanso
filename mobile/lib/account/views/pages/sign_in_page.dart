// Kanso — sign_in_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/kanso_routes.dart';
import '../../../core/router/settings_navigation.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../models/data/account_outcome.dart';
import '../../models/data/sign_in_mode.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import '../widgets/binding_question_listener.dart';
import '../widgets/choose_username_pane.dart';
import '../widgets/create_account_pane.dart';
import '../widgets/provider_code_pane.dart';
import '../widgets/sign_in_pane.dart';

/// Settings › Account › Sign in: signing in, making an account, and what
/// comes of either: a GitHub code, a username for a new identity, the
/// question about the notes already on this phone, and the recovery key of
/// the account just made.
class SignInPage extends StatelessWidget {
  const SignInPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SignInBloc>(
      create: (context) =>
          getIt<SignInBloc>()..add(const SignInEvent.started()),
      child: BindingQuestionListener<SignInBloc, SignInState>(
        questionOf: (state) => state.question,
        onAnswered: (context, answer) =>
            context.read<SignInBloc>().add(SignInEvent.bindingAnswered(answer)),
        child: BlocListener<SignInBloc, SignInState>(
          listenWhen: (before, after) =>
              before.outcome != after.outcome && after.outcome != null,
          listener: (context, state) {
            final router = GoRouter.of(context);
            final key = state.recoveryKey;
            if (state.outcome == AccountOutcome.locked) {
              router
                ..popToSettings()
                ..push<void>(KansoRoutes.settingsUnlock);
            } else if (key != null) {
              router.popToRecoveryKey(key);
            } else {
              router.popToSettings();
            }
          },
          child: Scaffold(
            backgroundColor: context.colors.app,
            body: BlocBuilder<SignInBloc, SignInState>(
              builder: (context, state) {
                final code = state.code;
                final suggested = state.suggestedUsername;
                final pane = code != null
                    ? ProviderCodePane(code: code)
                    : suggested != null
                    ? ChooseUsernamePane(state: state, suggested: suggested)
                    : state.mode == SignInMode.create
                    ? CreateAccountPane(state: state)
                    : SignInPane(
                        state: state,
                        onRecover: (server) =>
                            context.push(KansoRoutes.recover(server: server)),
                      );
                return PopScope(
                  canPop: state.mode == SignInMode.signIn && code == null,
                  onPopInvokedWithResult: (didPop, _) {
                    if (!didPop && state.busy == null) {
                      context.read<SignInBloc>().add(
                        const SignInEvent.modeChanged(SignInMode.signIn),
                      );
                    }
                  },
                  child: pane,
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}
