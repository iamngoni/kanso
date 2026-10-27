// Kanso — rust_account_repository_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: only_throw_errors

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/account.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/impl/rust_account_repository.dart';
import 'package:kanso_passkeys/kanso_passkeys.dart';
import 'package:kanso_passkeys/src/messages.g.dart' show PasskeysHostApi;
import 'package:mocktail/mocktail.dart';

class _MockAccount extends Mock implements KansoAccount {}

/// The platform’s passkey sheet, answering what it is told to.
class _Sheet extends PasskeysHostApi {
  PasskeyOutcome outcome = PasskeyOutcome.done;
  Object? failure;
  final List<({int ticket, String kind, String options})> asked =
      <({int ticket, String kind, String options})>[];

  @override
  Future<PasskeyOutcome> run(
    int ticket,
    String kind,
    String optionsJson,
  ) async {
    asked.add((ticket: ticket, kind: kind, options: optionsJson));
    final broken = failure;
    if (broken != null) {
      throw broken;
    }
    return outcome;
  }
}

void main() {
  late _MockAccount account;
  late RustAccountRepository repository;
  late StreamController<PasskeyRequest> requests;
  late _Sheet sheet;

  setUpAll(() => registerFallbackValue(PasskeyFailure.failed));

  setUp(() {
    account = _MockAccount();
    requests = StreamController<PasskeyRequest>();
    sheet = _Sheet();
    KansoPasskeys.debugApi = sheet;
    when(account.passkeyRequests).thenAnswer((_) => requests.stream);
    when(
      () => account.passkeyFailed(
        ticket: any(named: 'ticket'),
        failure: any(named: 'failure'),
      ),
    ).thenAnswer((_) async {});
    repository = RustAccountRepository(account);
  });

  tearDown(() async {
    await repository.dispose();
    unawaited(requests.close());
    KansoPasskeys.debugApi = PasskeysHostApi();
  });

  group('RustAccountRepository, the calls', () {
    test('the account now is what the library says', () async {
      when(account.state).thenAnswer(
        (_) async => const AccountState.signedIn(username: 'ana', server: 's'),
      );
      expect(
        await repository.state(),
        const Right<ApplicationError, AccountState>(
          AccountState.signedIn(username: 'ana', server: 's'),
        ),
      );
    });

    test(
      'a sign-in passes everything on, and the library’s error is mapped',
      () async {
        when(
          () => account.signInPassword(
            server: 's',
            username: 'ana',
            password: 'pw',
            deviceName: 'iPhone',
          ),
        ).thenAnswer(
          (_) => Future<AccountState>.error(
            const KansoError.auth(
              message:
                  'That username and password don’t '
                  'match.',
            ),
          ),
        );
        final result = await repository.signInPassword(
          server: 's',
          username: 'ana',
          password: 'pw',
          deviceName: 'iPhone',
        );
        expect(
          result,
          const Left<ApplicationError, AccountState>(
            ApplicationError.auth(
              message: 'That username and password don’t match.',
            ),
          ),
        );
      },
    );

    test(
      'a page that ended and a flow that was cancelled return unit',
      () async {
        when(
          () =>
              account.providerPageEnded(callbackUrl: 'kanso://auth/apple?x=1'),
        ).thenAnswer((_) async {});
        when(account.cancelProvider).thenAnswer((_) async {});
        expect(
          await repository.providerPageEnded('kanso://auth/apple?x=1'),
          const Right<ApplicationError, Unit>(unit),
        );
        expect(
          await repository.cancelProvider(),
          const Right<ApplicationError, Unit>(unit),
        );
      },
    );

    test('an answer to the question, or none, is passed on', () async {
      when(() => account.answerBinding(answer: BindingAnswer.add)).thenAnswer(
        (_) async => const AccountState.signedIn(username: 'ana', server: 's'),
      );
      when(() => account.answerBinding()).thenAnswer(
        (_) async => const AccountState.signedOut(),
      );
      expect(
        await repository.answerBinding(BindingAnswer.add),
        const Right<ApplicationError, AccountState>(
          AccountState.signedIn(username: 'ana', server: 's'),
        ),
      );
      expect(
        await repository.answerBinding(null),
        const Right<ApplicationError, AccountState>(AccountState.signedOut()),
      );
    });

    test('the recovery key kept, and saying it was saved', () async {
      when(account.pendingRecoveryKey).thenAnswer(
        (_) async => const RecoveryKeyText(key: 'ABCD-EFGH'),
      );
      when(account.recoveryKeySaved).thenAnswer((_) async {});
      expect(
        await repository.pendingRecoveryKey(),
        const Right<ApplicationError, RecoveryKeyText?>(
          RecoveryKeyText(key: 'ABCD-EFGH'),
        ),
      );
      expect(
        await repository.recoveryKeySaved(),
        const Right<ApplicationError, Unit>(unit),
      );
    });

    test('the streams of the library are passed on', () async {
      when(account.stateStream).thenAnswer(
        (_) => Stream<AccountState>.value(const AccountState.signedOut()),
      );
      when(() => account.requestApproval(deviceName: 'iPhone')).thenAnswer(
        (_) => Stream<ApprovalWait>.value(
          const ApprovalWait(
            phase: ApprovalPhase.waiting(),
            otherDevices: 1,
            offerRecovery: false,
          ),
        ),
      );
      expect(
        await repository.stateStream().toList(),
        <AccountState>[const AccountState.signedOut()],
      );
      expect(
        (await repository.requestApproval('iPhone').toList()).single.phase,
        const ApprovalPhase.waiting(),
      );
    });
  });

  group('RustAccountRepository, the passkey sheets', () {
    PasskeyRequest request(int ticket, PasskeyRequestKind kind) =>
        PasskeyRequest(ticket: ticket, kind: kind, optionsJson: 'options');

    Future<void> relay(PasskeyRequest next) async {
      requests.add(next);
      await pumpEventQueue();
    }

    test('a request is run by the plugin, as its kind', () async {
      repository.startRelayingPasskeys();
      await relay(request(1, PasskeyRequestKind.register));
      await relay(request(2, PasskeyRequestKind.assert_));
      expect(sheet.asked.map((ask) => ask.ticket), <int>[1, 2]);
      expect(sheet.asked.map((ask) => ask.kind), <String>[
        'register',
        'assert',
      ]);
      expect(sheet.asked.first.options, 'options');
      // The plugin gave the answer to Rust itself: nothing is said to it.
      verifyNever(
        () => account.passkeyFailed(
          ticket: any(named: 'ticket'),
          failure: any(named: 'failure'),
        ),
      );
    });

    test('starting twice listens once', () async {
      repository
        ..startRelayingPasskeys()
        ..startRelayingPasskeys();
      await relay(request(1, PasskeyRequestKind.assert_));
      expect(sheet.asked, hasLength(1));
      verify(account.passkeyRequests).called(1);
    });

    test('a sheet the person dismissed is told to the library', () async {
      sheet.outcome = PasskeyOutcome.cancelled;
      repository.startRelayingPasskeys();
      await relay(request(3, PasskeyRequestKind.assert_));
      verify(
        () =>
            account.passkeyFailed(ticket: 3, failure: PasskeyFailure.cancelled),
      ).called(1);
    });

    test(
      'a phone that cannot run it, and any other failure, are told too',
      () async {
        repository.startRelayingPasskeys();
        sheet.outcome = PasskeyOutcome.unavailable;
        await relay(request(4, PasskeyRequestKind.register));
        sheet.outcome = PasskeyOutcome.failed;
        await relay(request(5, PasskeyRequestKind.register));
        verify(
          () => account.passkeyFailed(
            ticket: 4,
            failure: PasskeyFailure.unavailable,
          ),
        ).called(1);
        verify(
          () =>
              account.passkeyFailed(ticket: 5, failure: PasskeyFailure.failed),
        ).called(1);
      },
    );

    test('a plugin that throws is a failure, not a lost ceremony', () async {
      sheet.failure = StateError('no activity');
      repository.startRelayingPasskeys();
      await relay(request(6, PasskeyRequestKind.assert_));
      verify(
        () => account.passkeyFailed(ticket: 6, failure: PasskeyFailure.failed),
      ).called(1);
    });

    test('a ceremony that ended meanwhile does not break the relay', () async {
      when(
        () => account.passkeyFailed(
          ticket: any(named: 'ticket'),
          failure: any(named: 'failure'),
        ),
      ).thenAnswer((_) => Future<void>.error(StateError('gone')));
      sheet.outcome = PasskeyOutcome.cancelled;
      repository.startRelayingPasskeys();
      await relay(request(7, PasskeyRequestKind.assert_));
      sheet.outcome = PasskeyOutcome.done;
      await relay(request(8, PasskeyRequestKind.assert_));
      expect(sheet.asked.map((ask) => ask.ticket), <int>[7, 8]);
    });

    test('after dispose requests are no longer run', () async {
      repository.startRelayingPasskeys();
      await repository.dispose();
      await relay(request(9, PasskeyRequestKind.assert_));
      expect(sheet.asked, isEmpty);
    });
  });
}
