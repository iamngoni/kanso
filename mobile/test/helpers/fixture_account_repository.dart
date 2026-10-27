// Kanso — fixture_account_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/account_repository.dart';

/// What a server that offers everything offers.
const ServerMethods allMethods = ServerMethods(
  signupsOpen: true,
  password: true,
  passkeys: true,
  providers: <ProviderKind>[
    ProviderKind.apple,
    ProviderKind.google,
    ProviderKind.github,
  ],
);

/// A recovery key as the desktop groups it.
const RecoveryKeyText sampleRecoveryKey = RecoveryKeyText(
  key: 'K7QF-2MXD-9PWA-HC4T-ZR6N-B8VE-3YJL',
);

/// An account service for widget tests: the account it is told it is, a
/// server that offers what the test says, and the calls it was asked for
/// recorded. A call waits for [gate] when there is one.
class FixtureAccountRepository implements AccountRepository {
  FixtureAccountRepository({this.current = const AccountState.signedOut()});

  /// The account now.
  AccountState current;

  /// What the server offers.
  Either<ApplicationError, ServerMethods> methods =
      const Right<ApplicationError, ServerMethods>(allMethods);

  /// What a call that signs in answers.
  Either<ApplicationError, AccountState> signedIn =
      const Right<ApplicationError, AccountState>(
        AccountState.signedIn(
          username: 'ana',
          server: 'https://example.invalid',
        ),
      );

  /// What a call that makes an account answers.
  Either<ApplicationError, SignedUp> signedUp =
      const Right<ApplicationError, SignedUp>(
        SignedUp(
          state: AccountState.signedIn(
            username: 'ana',
            server: 'https://example.invalid',
          ),
          recoveryKey: sampleRecoveryKey,
        ),
      );

  /// The key the library kept, unsaved.
  RecoveryKeyText? pending;

  /// The steps of the provider flow a call starts.
  List<ProviderEvent> providerSteps = const <ProviderEvent>[];

  /// The waits a request for approval goes through.
  List<ApprovalWait> approvalWaits = const <ApprovalWait>[];

  /// When set, a call that signs in waits for it.
  Completer<void>? gate;

  /// The calls asked for, by name.
  final List<String> calls = <String>[];

  final StreamController<AccountState> _changes =
      StreamController<AccountState>.broadcast();

  /// Tells the listeners the account is [next].
  void change(AccountState next) {
    current = next;
    _changes.add(next);
  }

  Future<void> _wait() async => gate?.future;

  @override
  Future<Either<ApplicationError, AccountState>> state() async =>
      Right<ApplicationError, AccountState>(current);

  @override
  Stream<AccountState> stateStream() {
    final watched = StreamController<AccountState>()..add(current);
    final changes = _changes.stream.listen(watched.add);
    watched.onCancel = changes.cancel;
    return watched.stream;
  }

  @override
  Future<Either<ApplicationError, ServerMethods>> serverMethods(
    String server,
  ) async {
    calls.add('serverMethods $server');
    return methods;
  }

  @override
  Future<Either<ApplicationError, SignedUp>> signUpPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) async {
    calls.add('signUpPassword $username');
    await _wait();
    return signedUp;
  }

  @override
  Future<Either<ApplicationError, AccountState>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) async {
    calls.add('signInPassword $username');
    await _wait();
    return signedIn;
  }

  @override
  Future<Either<ApplicationError, AccountState>> answerBinding(
    BindingAnswer? answer,
  ) async {
    calls.add('answerBinding $answer');
    return signedIn;
  }

  @override
  Stream<ProviderEvent> startProvider({
    required String server,
    required ProviderKind provider,
    required String deviceName,
  }) {
    calls.add('startProvider $provider');
    return Stream<ProviderEvent>.fromIterable(providerSteps);
  }

  @override
  Future<Either<ApplicationError, Unit>> providerPageEnded(
    String? callbackUrl,
  ) async {
    calls.add('providerPageEnded $callbackUrl');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> cancelProvider() async {
    calls.add('cancelProvider');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, SignedUp>> finishProviderSignUp(
    String username,
  ) async {
    calls.add('finishProviderSignUp $username');
    return signedUp;
  }

  @override
  Future<Either<ApplicationError, AccountState>> passkeySignIn({
    required String server,
    required String deviceName,
  }) async {
    calls.add('passkeySignIn');
    return signedIn;
  }

  @override
  Future<Either<ApplicationError, SignedUp>> passkeySignUp({
    required String server,
    required String username,
    required String deviceName,
  }) async {
    calls.add('passkeySignUp $username');
    return signedUp;
  }

  @override
  Future<Either<ApplicationError, AccountState>> unlockWithPassword(
    String password,
  ) async {
    calls.add('unlockWithPassword');
    await _wait();
    return signedIn;
  }

  @override
  Stream<ApprovalWait> requestApproval(String deviceName) {
    calls.add('requestApproval');
    return Stream<ApprovalWait>.fromIterable(approvalWaits);
  }

  @override
  Future<Either<ApplicationError, Unit>> cancelApproval() async {
    calls.add('cancelApproval');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, SignedUp>> recover({
    required String server,
    required String username,
    required String recoveryKey,
    required String newPassword,
    required String deviceName,
  }) async {
    calls.add('recover $username');
    await _wait();
    return signedUp;
  }

  @override
  Future<Either<ApplicationError, AccountState>> cancelLocked() async {
    calls.add('cancelLocked');
    return const Right<ApplicationError, AccountState>(
      AccountState.signedOut(),
    );
  }

  @override
  Future<Either<ApplicationError, RecoveryKeyText?>>
  pendingRecoveryKey() async {
    calls.add('pendingRecoveryKey');
    return Right<ApplicationError, RecoveryKeyText?>(pending);
  }

  @override
  Future<Either<ApplicationError, Unit>> recoveryKeySaved() async {
    calls.add('recoveryKeySaved');
    pending = null;
    return const Right<ApplicationError, Unit>(unit);
  }
}
