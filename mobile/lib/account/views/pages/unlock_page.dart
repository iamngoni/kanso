// Kanso — unlock_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/kanso_routes.dart';
import '../../../core/router/settings_navigation.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/device_name.dart';
import '../../../injection.dart';
import '../../state/unlock/unlock_bloc.dart';
import '../widgets/approval_wait_pane.dart';
import '../widgets/binding_question_listener.dart';
import '../widgets/unlock_pane.dart';

/// Settings › Account › Open your notes on this device: for an account that
/// is signed in on this phone, which has no key for its notes. The password,
/// another device's approval (a screen of its own while it waits), the
/// recovery key, or signing out.
class UnlockPage extends StatelessWidget {
  const UnlockPage({super.key});

  @override
  Widget build(BuildContext context) {
    final info = context.read<AccountBloc>().state.locked;
    final deviceName = deviceNameOf(context);
    return BlocProvider<UnlockBloc>(
      create: (context) => getIt<UnlockBloc>()
        ..add(
          UnlockEvent.started(
            resumeApproval: info?.approvalPending ?? false,
            deviceName: deviceName,
          ),
        ),
      child: MultiBlocListener(
        listeners: <BlocListener<dynamic, dynamic>>[
          BlocListener<UnlockBloc, UnlockState>(
            listenWhen: (before, after) =>
                before.outcome != after.outcome && after.outcome != null,
            listener: (context, state) => GoRouter.of(context).popToSettings(),
          ),
          // The account is not locked any more, however that came about: the
          // person is done here.
          BlocListener<AccountBloc, AccountBlocState>(
            listenWhen: (before, after) =>
                before.locked != null && after.locked == null,
            listener: (context, state) => GoRouter.of(context).popToSettings(),
          ),
        ],
        child: BindingQuestionListener<UnlockBloc, UnlockState>(
          questionOf: (state) => state.question,
          onAnswered: (context, answer) => context.read<UnlockBloc>().add(
            UnlockEvent.bindingAnswered(answer),
          ),
          child: Scaffold(
            backgroundColor: context.colors.app,
            body: BlocBuilder<AccountBloc, AccountBlocState>(
              buildWhen: (before, after) => before.locked != after.locked,
              builder: (context, account) {
                final locked = account.locked;
                if (locked == null) {
                  return const SizedBox.shrink();
                }
                return BlocBuilder<UnlockBloc, UnlockState>(
                  builder: (context, state) {
                    void recover() => context.push(
                      KansoRoutes.recover(server: locked.server),
                    );
                    final wait = state.approval;
                    if (wait != null) {
                      return ApprovalWaitPane(wait: wait, onRecover: recover);
                    }
                    return UnlockPane(
                      state: state,
                      info: locked,
                      onRecover: recover,
                    );
                  },
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}
