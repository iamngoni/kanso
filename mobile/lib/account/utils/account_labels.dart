// Kanso — account_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:intl/intl.dart';

import '../../core/bridge/api/types.dart';
import '../../gen/l10n/app_localizations.dart';
import '../models/data/account_notice.dart';
import '../models/data/form_problem.dart';

/// The words of the account screens.
extension AccountLabels on AppLocalizations {
  /// What is wrong with what was typed.
  String formProblem(FormProblem problem) {
    return switch (problem) {
      FormProblem.usernameInvalid => formProblemUsernameInvalid,
      FormProblem.usernameReserved => formProblemUsernameReserved,
      FormProblem.passwordTooShort => formProblemPasswordTooShort,
      FormProblem.passwordTooLong => formProblemPasswordTooLong,
      FormProblem.passwordSameAsUsername => formProblemPasswordSameAsUsername,
      FormProblem.passwordsDiffer => formProblemPasswordsDiffer,
      FormProblem.credentialsDontMatch => formProblemCredentialsDontMatch,
      FormProblem.recoveryKeyMissing => formProblemRecoveryKeyMissing,
    };
  }

  /// The name of a provider.
  String providerName(ProviderKind provider) {
    return switch (provider) {
      ProviderKind.apple => accountProviderApple,
      ProviderKind.google => accountProviderGoogle,
      ProviderKind.github => accountProviderGithub,
    };
  }

  /// The button that signs in with a provider.
  String continueWith(ProviderKind provider) {
    return switch (provider) {
      ProviderKind.apple => accountContinueWithApple,
      ProviderKind.google => accountContinueWithGoogle,
      ProviderKind.github => accountContinueWithGithub,
    };
  }

  /// What the wait for another device's approval says of [phase].
  String approvalPhase(ApprovalPhase phase) {
    return switch (phase) {
      ApprovalPhase_Requesting() => accountApprovalRequesting,
      ApprovalPhase_Waiting() ||
      ApprovalPhase_Compare() => accountApprovalWaiting,
      ApprovalPhase_Unlocking() => accountApprovalUnlocking,
      ApprovalPhase_Approved() => accountApprovalApproved,
      ApprovalPhase_NoDevices() => accountApprovalNoDevices,
      ApprovalPhase_Cancelled() => accountApprovalCancelled,
      ApprovalPhase_Expired() => accountApprovalExpired,
      ApprovalPhase_Failed(:final message) =>
        message.trim().isEmpty ? accountApprovalFailed : message,
    };
  }

  /// What the Account page says once a change went through.
  String accountNotice(AccountNotice notice) {
    return switch (notice) {
      AccountNoticeDeviceSignedOut(:final name) => accountNoticeDeviceSignedOut(
        name,
      ),
      AccountNoticePasskeyAdded(:final canUnlock) =>
        canUnlock
            ? accountNoticePasskeyAddedCanUnlock
            : accountNoticePasskeyAddedSignsIn,
      AccountNoticePasskeyRemoved() => accountNoticePasskeyRemoved,
      AccountNoticeProviderLinked() => accountNoticeProviderLinked,
      AccountNoticeProviderUnlinked() => accountNoticeProviderUnlinked,
    };
  }

  /// A server's address as a row shows it: the default one by its name, any
  /// other by its host.
  String serverName(String server, {required String defaultServer}) {
    if (server.isEmpty || server == defaultServer) {
      return accountServerDefault;
    }
    return hostOf(server);
  }
}

/// An address without its scheme and trailing slash, for a row's value.
String hostOf(String server) {
  final trimmed = server.trim();
  final at = trimmed.indexOf('://');
  final rest = at < 0 ? trimmed : trimmed.substring(at + 3);
  return rest.endsWith('/') ? rest.substring(0, rest.length - 1) : rest;
}

/// A day as the Account page writes it ("Oct 26, 2026"), in the local zone.
String accountDate(BuildContext context, int millisecondsSinceEpoch) {
  final locale = Localizations.localeOf(context).toString();
  return DateFormat.yMMMd(
    locale,
  ).format(DateTime.fromMillisecondsSinceEpoch(millisecondsSinceEpoch));
}
