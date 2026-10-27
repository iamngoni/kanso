// Kanso — recover_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/settings_navigation.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/recover/recover_bloc.dart';
import '../widgets/binding_question_listener.dart';
import '../widgets/recover_pane.dart';

/// Settings › Account › Reset your password: signing in with the recovery
/// key. For [server] when the phone is signed out; a phone locked to an
/// account recovers that account, whatever it is told.
class RecoverPage extends StatelessWidget {
  const RecoverPage({this.server, super.key});

  /// The server of the account; the default one when null.
  final String? server;

  @override
  Widget build(BuildContext context) {
    final locked = context.read<AccountBloc>().state.locked;
    return BlocProvider<RecoverBloc>(
      create: (context) =>
          getIt<RecoverBloc>()
            ..add(RecoverEvent.started(server: locked?.server ?? server)),
      child: BindingQuestionListener<RecoverBloc, RecoverState>(
        questionOf: (state) => state.question,
        onAnswered: (context, answer) => context.read<RecoverBloc>().add(
          RecoverEvent.bindingAnswered(answer),
        ),
        child: BlocListener<RecoverBloc, RecoverState>(
          listenWhen: (before, after) =>
              before.outcome != after.outcome && after.outcome != null,
          listener: (context, state) {
            final router = GoRouter.of(context);
            final key = state.recoveryKey;
            if (key != null) {
              router.popToRecoveryKey(key);
            } else {
              router.popToSettings();
            }
          },
          child: Scaffold(
            backgroundColor: context.colors.app,
            body: BlocBuilder<RecoverBloc, RecoverState>(
              builder: (context, state) =>
                  RecoverPane(state: state, fixedUsername: locked?.username),
            ),
          ),
        ),
      ),
    );
  }
}
