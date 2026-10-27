// Kanso — account_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';
import 'sync_suite.dart' show unreachableServer;

const String _password = 'correct horse battery';

/// Signing in and up, the locked account, recovery and device approval as
/// the new device, through the bridge: every call of `KansoAccount`, against
/// no server.
void accountSuite(OpenLibrary open) {
  group('KansoAccount', () {
    test('a new library is signed out, and the stream starts there', () async {
      final account = (await open()).account();
      expect(await account.state(), const AccountState.signedOut());
      final first = await account.stateStream().first.timeout(
        const Duration(seconds: 10),
      );
      expect(first, const AccountState.signedOut());
    });

    test('a server that is not there is a network error', () async {
      final account = (await open()).account();
      await expectLater(
        account.serverMethods(server: unreachableServer),
        throwsA(isA<KansoError_Network>()),
      );
      await expectLater(
        account.signUpPassword(
          server: unreachableServer,
          username: 'ada',
          password: _password,
          deviceName: 'Test phone',
        ),
        throwsA(isA<KansoError_Network>()),
      );
      await expectLater(
        account.signInPassword(
          server: unreachableServer,
          username: 'ada',
          password: _password,
          deviceName: 'Test phone',
        ),
        throwsA(isA<KansoError_Network>()),
      );
      await expectLater(
        account.passkeySignIn(
          server: unreachableServer,
          deviceName: 'Test phone',
        ),
        throwsA(isA<KansoError_Network>()),
      );
      expect(await account.state(), const AccountState.signedOut());
    });

    test('a passkey sign-up with no server is a network error', () async {
      final account = (await open()).account();
      await expectLater(
        account.passkeySignUp(
          server: unreachableServer,
          username: 'ada',
          deviceName: 'Test phone',
        ),
        throwsA(isA<KansoError_Network>()),
      );
    });

    test(
      'a recovery key that is not one is refused before any request',
      () async {
        final account = (await open()).account();
        await expectLater(
          account.recover(
            server: unreachableServer,
            username: 'ada',
            recoveryKey: 'AAAA',
            newPassword: _password,
            deviceName: 'Test phone',
          ),
          throwsA(isA<KansoError_Invalid>()),
        );
      },
    );

    test('nothing waits to be unlocked, answered or finished', () async {
      final account = (await open()).account();
      await expectLater(
        account.unlockWithPassword(password: _password),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        account.answerBinding(answer: BindingAnswer.add),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        account.answerBinding(),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        account.finishProviderSignUp(username: 'ada'),
        throwsA(isA<KansoError_Invalid>()),
      );
      expect(
        await account.cancelLocked(),
        const AccountState.signedOut(),
      );
    });

    test('no recovery key is waiting to be written down', () async {
      final account = (await open()).account();
      expect(await account.pendingRecoveryKey(), isNull);
      await account.recoveryKeySaved();
    });

    test('a provider sign-in against no server ends in a failure', () async {
      final account = (await open()).account();
      final events = account.startProvider(
        server: unreachableServer,
        provider: ProviderKind.github,
        deviceName: 'Test phone',
      );
      final first = await events.first.timeout(const Duration(seconds: 60));
      expect(first, isA<ProviderEvent_Failed>());
      await account.providerPageEnded();
      await account.cancelProvider();
    });

    test('a passkey the phone could not make is reported to no one', () async {
      final account = (await open()).account();
      final requests = <PasskeyRequest>[];
      final listening = account.passkeyRequests().listen(requests.add);
      await account.passkeyFailed(
        ticket: 1,
        failure: PasskeyFailure.cancelled,
      );
      await Future<void>.delayed(const Duration(milliseconds: 200));
      // Cancelling waits for the library's next event: don't wait for it.
      unawaited(listening.cancel());
      expect(requests, isEmpty);
    });

    test(
      'asking another device to approve, signed out, ends at once',
      () async {
        final account = (await open()).account();
        final first = await account
            .requestApproval(deviceName: 'Test phone')
            .first
            .timeout(const Duration(seconds: 10));
        expect(first.phase, isA<ApprovalPhase_Failed>());
        await account.cancelApproval();
      },
    );
  });
}
