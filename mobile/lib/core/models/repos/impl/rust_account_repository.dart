// Kanso — rust_account_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_passkeys/kanso_passkeys.dart';

import '../../../bridge/api/account.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/account_repository.dart';

/// [AccountRepository] over the Rust core's account.
///
/// It also carries the passkey sheets: whenever the core asks for one, the
/// platform's plugin runs it, and when the plugin could not (the person
/// dismissed it, the phone can't, anything else) the core is told why, so a
/// ceremony never waits on a lost answer. The sheet's answer itself goes
/// from the plugin straight to Rust, past Dart.
@LazySingleton(as: AccountRepository)
class RustAccountRepository implements AccountRepository {
  RustAccountRepository(this._account);

  final KansoAccount _account;

  // Cancelled in `dispose`; the repository lives as long as the app does.
  // ignore: cancel_subscriptions
  StreamSubscription<PasskeyRequest>? _passkeys;

  /// Starts carrying the passkey sheets. The app does it as it starts, and
  /// doing it again changes nothing.
  @PostConstruct()
  void startRelayingPasskeys() {
    _passkeys ??= cancelableStream(
      _account.passkeyRequests(),
    ).listen(_relay, onError: (Object _) {});
  }

  /// Stops carrying the passkey sheets. The app never does: this is for
  /// tests.
  Future<void> dispose() async {
    final listening = _passkeys;
    _passkeys = null;
    await listening?.cancel();
  }

  Future<void> _relay(PasskeyRequest request) async {
    final kind = switch (request.kind) {
      PasskeyRequestKind.register => 'register',
      PasskeyRequestKind.assert_ => 'assert',
    };
    PasskeyOutcome outcome;
    try {
      outcome = await KansoPasskeys.run(
        request.ticket,
        kind,
        request.optionsJson,
      );
    } on Object {
      outcome = PasskeyOutcome.failed;
    }
    final failure = switch (outcome) {
      PasskeyOutcome.done => null,
      PasskeyOutcome.cancelled => PasskeyFailure.cancelled,
      PasskeyOutcome.unavailable => PasskeyFailure.unavailable,
      PasskeyOutcome.failed => PasskeyFailure.failed,
    };
    if (failure == null) {
      return;
    }
    try {
      await _account.passkeyFailed(ticket: request.ticket, failure: failure);
    } on Object {
      // The ceremony ended on its own side meanwhile.
    }
  }

  @override
  Future<Either<ApplicationError, AccountState>> state() =>
      attempt(_account.state);

  @override
  Stream<AccountState> stateStream() =>
      cancelableStream(_account.stateStream());

  @override
  Future<Either<ApplicationError, ServerMethods>> serverMethods(
    String server,
  ) => attempt(() => _account.serverMethods(server: server));

  @override
  Future<Either<ApplicationError, SignedUp>> signUpPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) => attempt(
    () => _account.signUpPassword(
      server: server,
      username: username,
      password: password,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, AccountState>> signInPassword({
    required String server,
    required String username,
    required String password,
    required String deviceName,
  }) => attempt(
    () => _account.signInPassword(
      server: server,
      username: username,
      password: password,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, AccountState>> answerBinding(
    BindingAnswer? answer,
  ) => attempt(() => _account.answerBinding(answer: answer));

  @override
  Stream<ProviderEvent> startProvider({
    required String server,
    required ProviderKind provider,
    required String deviceName,
  }) => cancelableStream(
    _account.startProvider(
      server: server,
      provider: provider,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, Unit>> providerPageEnded(
    String? callbackUrl,
  ) => attempt(() async {
    await _account.providerPageEnded(callbackUrl: callbackUrl);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> cancelProvider() => attempt(() async {
    await _account.cancelProvider();
    return unit;
  });

  @override
  Future<Either<ApplicationError, SignedUp>> finishProviderSignUp(
    String username,
  ) => attempt(() => _account.finishProviderSignUp(username: username));

  @override
  Future<Either<ApplicationError, AccountState>> passkeySignIn({
    required String server,
    required String deviceName,
  }) => attempt(
    () => _account.passkeySignIn(server: server, deviceName: deviceName),
  );

  @override
  Future<Either<ApplicationError, SignedUp>> passkeySignUp({
    required String server,
    required String username,
    required String deviceName,
  }) => attempt(
    () => _account.passkeySignUp(
      server: server,
      username: username,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, AccountState>> unlockWithPassword(
    String password,
  ) => attempt(() => _account.unlockWithPassword(password: password));

  @override
  Stream<ApprovalWait> requestApproval(String deviceName) =>
      cancelableStream(_account.requestApproval(deviceName: deviceName));

  @override
  Future<Either<ApplicationError, Unit>> cancelApproval() => attempt(() async {
    await _account.cancelApproval();
    return unit;
  });

  @override
  Future<Either<ApplicationError, SignedUp>> recover({
    required String server,
    required String username,
    required String recoveryKey,
    required String newPassword,
    required String deviceName,
  }) => attempt(
    () => _account.recover(
      server: server,
      username: username,
      recoveryKey: recoveryKey,
      newPassword: newPassword,
      deviceName: deviceName,
    ),
  );

  @override
  Future<Either<ApplicationError, AccountState>> cancelLocked() =>
      attempt(_account.cancelLocked);

  @override
  Future<Either<ApplicationError, RecoveryKeyText?>> pendingRecoveryKey() =>
      attempt(_account.pendingRecoveryKey);

  @override
  Future<Either<ApplicationError, Unit>> recoveryKeySaved() =>
      attempt(() async {
        await _account.recoveryKeySaved();
        return unit;
      });
}
