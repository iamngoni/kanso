// Kanso — account_actions.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/bridge/api/types.dart';
import '../../core/router/kanso_routes.dart';
import '../../core/utils/context_extensions.dart';
import '../models/data/account_action_field.dart';
import '../state/account_details/account_details_bloc.dart';
import '../views/widgets/account_action_sheet.dart';
import 'account_labels.dart';

/// The changes to the account the Account page offers: each asks in a sheet
/// for what it needs and, once that is given, tells the page's
/// [AccountDetailsBloc] to make it. It needs one above the `context` given.
abstract final class AccountActions {
  /// Shows the recovery key waiting to be saved (the one a change made, or
  /// else the one the library kept), and reads the account again once the
  /// person is done with it.
  static Future<void> showRecoveryKey(
    BuildContext context, {
    RecoveryKeyText? key,
  }) async {
    final bloc = context.read<AccountDetailsBloc>();
    await GoRouter.of(
      context,
    ).push<void>(KansoRoutes.settingsRecoveryKey, extra: key);
    if (!bloc.isClosed) {
      bloc.add(const AccountDetailsEvent.started());
    }
  }

  /// Signs another device out, after asking.
  static Future<void> revokeDevice(
    BuildContext context,
    DeviceRow device,
  ) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountRevokeTitle(device.title),
      message: l10n.accountRevokeMessage,
      confirmLabel: l10n.accountDeviceSignOut,
      danger: true,
    );
    if (values != null) {
      bloc.add(AccountDetailsEvent.deviceRevoked(device.id));
    }
  }

  /// Signs this phone out, after asking. A recovery key not yet saved is
  /// warned of, as signing out deletes it.
  static Future<void> signOut(
    BuildContext context, {
    required String username,
    required bool keyUnsaved,
  }) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountSignOutTitle(username),
      message: l10n.accountSignOutMessage,
      warning: keyUnsaved ? l10n.accountSignOutUnsavedKey : null,
      confirmLabel: l10n.accountConfirmSignOut,
      danger: true,
    );
    if (values != null) {
      bloc.add(const AccountDetailsEvent.signedOut());
    }
  }

  /// Changes the password: the current one, and a new one twice.
  static Future<void> changePassword(BuildContext context) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountChangePasswordTitle,
      message: l10n.accountChangePasswordMessage,
      confirmLabel: l10n.accountChangePasswordTitle,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
        AccountActionField(label: l10n.accountNewPassword),
        AccountActionField(label: l10n.accountRepeatPassword),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.passwordChanged(
          current: values[0],
          newPassword: values[1],
          repeat: values[2],
        ),
      );
    }
  }

  /// Makes a new recovery key, with the current password.
  static Future<void> newRecoveryKey(BuildContext context) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountNewRecoveryKeyTitle,
      message: l10n.accountNewRecoveryKeyMessage,
      confirmLabel: l10n.accountNewRecoveryKeyButton,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.recoveryKeyRequested(currentPassword: values[0]),
      );
    }
  }

  /// Deletes the account, with its password and its name typed.
  static Future<void> deleteAccount(
    BuildContext context, {
    required String username,
  }) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountDeleteTitle,
      message: l10n.accountDeleteMessage(username),
      confirmLabel: l10n.accountDeleteAccount,
      danger: true,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
        AccountActionField(
          label: l10n.accountDeleteTypeUsername(username),
          obscure: false,
        ),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.accountDeleted(
          password: values[0],
          typedUsername: values[1],
        ),
      );
    }
  }

  /// Adds a passkey, with the current password.
  static Future<void> addPasskey(BuildContext context) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountAddPasskeyTitle,
      message: l10n.accountAddPasskeyMessage,
      confirmLabel: l10n.accountPasskeyAdd,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
      ],
    );
    if (values != null) {
      bloc.add(AccountDetailsEvent.passkeyAdded(currentPassword: values[0]));
    }
  }

  /// Removes a passkey, with the current password.
  static Future<void> removePasskey(
    BuildContext context,
    PasskeyRow passkey,
  ) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountRemovePasskeyTitle(passkey.label),
      message: l10n.accountRemovePasskeyMessage,
      confirmLabel: l10n.accountRemovePasskeyButton,
      danger: true,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.passkeyRemoved(
          id: passkey.id,
          currentPassword: values[0],
        ),
      );
    }
  }

  /// Links Apple, Google or GitHub, with the current password.
  static Future<void> linkProvider(
    BuildContext context,
    ProviderKind provider,
  ) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountLinkTitle(l10n.providerName(provider)),
      message: l10n.accountLinkMessage,
      confirmLabel: l10n.accountProviderLink,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.providerLinked(
          provider: provider,
          currentPassword: values[0],
        ),
      );
    }
  }

  /// Unlinks Apple, Google or GitHub, with the current password.
  static Future<void> unlinkProvider(
    BuildContext context,
    ProviderKind provider,
  ) async {
    final l10n = context.l10n;
    final bloc = context.read<AccountDetailsBloc>();
    final values = await AccountActionSheet.show(
      context,
      title: l10n.accountUnlinkTitle(l10n.providerName(provider)),
      message: l10n.accountUnlinkMessage,
      confirmLabel: l10n.accountProviderUnlink,
      danger: true,
      fields: <AccountActionField>[
        AccountActionField(label: l10n.accountCurrentPassword),
      ],
    );
    if (values != null) {
      bloc.add(
        AccountDetailsEvent.providerUnlinked(
          provider: provider,
          currentPassword: values[0],
        ),
      );
    }
  }
}
