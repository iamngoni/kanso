// Kanso — account_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/router/settings_navigation.dart';
import '../../../core/state/account/account_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/account_details/account_details_bloc.dart';
import '../../utils/account_actions.dart';
import '../widgets/account_pane.dart';

/// Settings › Account: the page of the account that is signed in.
///
/// It leaves for Settings when the account ends here (signed out, deleted,
/// or ended by the server), and shows the recovery key of a change that made
/// one, then reads the account again.
class AccountPage extends StatelessWidget {
  const AccountPage({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AccountDetailsBloc>(
      create: (context) =>
          getIt<AccountDetailsBloc>()..add(const AccountDetailsEvent.started()),
      child: MultiBlocListener(
        listeners: <BlocListener<dynamic, dynamic>>[
          BlocListener<AccountDetailsBloc, AccountDetailsState>(
            listenWhen: (before, after) =>
                before.outcome != after.outcome && after.outcome != null,
            listener: (context, state) => GoRouter.of(context).popToSettings(),
          ),
          BlocListener<AccountDetailsBloc, AccountDetailsState>(
            listenWhen: (before, after) =>
                before.newKey == null && after.newKey != null,
            listener: (context, state) {
              context.read<AccountDetailsBloc>().add(
                const AccountDetailsEvent.keyShown(),
              );
              AccountActions.showRecoveryKey(context, key: state.newKey);
            },
          ),
          BlocListener<AccountBloc, AccountBlocState>(
            listenWhen: (before, after) =>
                before.signedIn && !after.signedIn && after.account != null,
            listener: (context, state) => GoRouter.of(context).popToSettings(),
          ),
        ],
        child: Scaffold(
          backgroundColor: context.colors.app,
          body: const AccountPane(),
        ),
      ),
    );
  }
}
