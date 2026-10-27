// Kanso — account_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/load_status.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../state/account_details/account_details_bloc.dart';
import 'account_pane_body.dart';
import 'account_pane_frame.dart';
import 'account_pane_loading.dart';

/// Account: who is signed in and on which server, the account's devices,
/// its ways to sign in (the password, passkeys, Apple, Google, GitHub), the
/// recovery key, the storage it uses, and signing out or deleting it. It
/// needs an [AccountDetailsBloc] above it.
class AccountPane extends StatelessWidget {
  const AccountPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return AccountPaneFrame(
      title: l10n.accountTitle,
      children: <Widget>[
        BlocBuilder<AccountDetailsBloc, AccountDetailsState>(
          builder: (context, state) {
            final details = state.details;
            return switch (state.status) {
              LoadStatus.loading => const AccountPaneLoading(),
              LoadStatus.failed => Center(
                child: KansoMessageCard(
                  title: l10n.accountLoadFailedTitle,
                  message: state.loadError == null
                      ? l10n.accountLoadFailedMessage
                      : l10n.accountError(state.loadError!),
                  actions: <Widget>[
                    KansoButton(
                      label: l10n.accountRetry,
                      onPressed: () => context.read<AccountDetailsBloc>().add(
                        const AccountDetailsEvent.started(),
                      ),
                    ),
                  ],
                ),
              ),
              LoadStatus.ready when details != null => AccountPaneBody(
                state: state,
                details: details,
              ),
              LoadStatus.ready => const SizedBox.shrink(),
            };
          },
        ),
      ],
    );
  }
}
