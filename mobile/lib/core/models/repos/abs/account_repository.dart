// Kanso — account_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Signing in and up, the locked account, recovery and device approval as
/// the new device. The keys and the session stay in the phone's secret
/// store, read by the Rust core; only a recovery key the person must write
/// down comes through.
abstract interface class AccountRepository {
  /// The account now.
  Future<Either<ApplicationError, AccountState>> state();

  /// The account now and each time it changes.
  Stream<AccountState> stateStream();

  /// How the server at [server] lets a person in.
  Future<Either<ApplicationError, ServerMethods>> serverMethods(String server);

  Future<Either<ApplicationError, SignedUp>> signUpPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  });

  Future<Either<ApplicationError, AccountState>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  });

  /// Answers the question a sign-in asked about the notes on this phone; a
  /// null [answer] cancels.
  Future<Either<ApplicationError, AccountState>> answerBinding(
    BindingAnswer? answer,
  );

  /// The steps of signing in with [provider], until the flow ends.
  Stream<ProviderEvent> startProvider({
    required String server,
    required ProviderKind provider,
    required String deviceName,
  });

  /// The provider's page ended at [callbackUrl] (Apple), or was dismissed
  /// (null).
  Future<Either<ApplicationError, Unit>> providerPageEnded(String? callbackUrl);

  Future<Either<ApplicationError, Unit>> cancelProvider();

  /// Makes the account for the identity a provider vouched for.
  Future<Either<ApplicationError, SignedUp>> finishProviderSignUp(
    String username,
  );

  Future<Either<ApplicationError, AccountState>> passkeySignIn({
    required String server,
    required String deviceName,
  });

  Future<Either<ApplicationError, SignedUp>> passkeySignUp({
    required String server,
    required String username,
    required String deviceName,
  });

  Future<Either<ApplicationError, AccountState>> unlockWithPassword(
    String password,
  );

  /// The wait for another device to approve this one, until it ends.
  Stream<ApprovalWait> requestApproval(String deviceName);

  Future<Either<ApplicationError, Unit>> cancelApproval();

  /// Signs in with the recovery key and sets a new password.
  Future<Either<ApplicationError, SignedUp>> recover({
    required String server,
    required String username,
    required String recoveryKey,
    required String newPassword,
    required String deviceName,
  });

  /// Ends a locked sign-in.
  Future<Either<ApplicationError, AccountState>> cancelLocked();

  /// The recovery key shown last that has not been confirmed saved.
  Future<Either<ApplicationError, RecoveryKeyText?>> pendingRecoveryKey();

  /// The person said they saved the recovery key shown last.
  Future<Either<ApplicationError, Unit>> recoveryKeySaved();
}
