// Kanso — recover_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/account_outcome.dart';
import 'package:kanso/account/models/data/form_problem.dart';
import 'package:kanso/account/state/recover/recover_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/mock_repositories.dart';

const String _server = 'https://sync.example.invalid';
const String _key = 'K7QF-2MXD-9PWA-HC4T-ZR6N-B8VE-3YJL';
const String _password = 'correct horse battery';
const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: _server,
);

void main() {
  late MockAccountRepository account;
  late MockSyncRepository sync;

  setUp(() {
    account = MockAccountRepository();
    sync = MockSyncRepository();
    when(
      sync.defaultServer,
    ).thenAnswer((_) async => const Right<ApplicationError, String>(_server));
  });

  Future<RecoverBloc> started({String? server}) async {
    final bloc = RecoverBloc(account, sync)
      ..add(RecoverEvent.started(server: server));
    await bloc.stream.firstWhere((state) => state.server.isNotEmpty);
    return bloc;
  }

  RecoverEvent submit({
    String username = 'ana',
    String recoveryKey = _key,
    String password = _password,
    String? repeat,
  }) {
    return RecoverEvent.submitted(
      username: username,
      recoveryKey: recoveryKey,
      password: password,
      repeat: repeat ?? password,
      deviceName: 'iPhone',
    );
  }

  group('RecoverBloc', () {
    test('uses the server it is told, else the default one', () async {
      expect(
        (await started(server: 'https://other.invalid')).state.server,
        'https://other.invalid',
      );
      expect((await started()).state.server, _server);
    });

    test('the form is checked before anything is sent', () async {
      final bloc = await started();
      Future<FormProblem?> problemOf(RecoverEvent event) async {
        bloc.add(event);
        await pumpEventQueue();
        return bloc.state.problem;
      }

      expect(
        await problemOf(submit(username: '!')),
        FormProblem.usernameInvalid,
      );
      expect(
        await problemOf(submit(recoveryKey: '  ')),
        FormProblem.recoveryKeyMissing,
      );
      expect(
        await problemOf(submit(password: 'short')),
        FormProblem.passwordTooShort,
      );
      expect(
        await problemOf(submit(repeat: 'different password')),
        FormProblem.passwordsDiffer,
      );
      verifyNever(
        () => account.recover(
          server: any(named: 'server'),
          username: any(named: 'username'),
          recoveryKey: any(named: 'recoveryKey'),
          newPassword: any(named: 'newPassword'),
          deviceName: any(named: 'deviceName'),
        ),
      );
      await bloc.close();
    });

    test('a recovery signs in and has a new key to write down', () async {
      when(
        () => account.recover(
          server: _server,
          username: 'ana',
          recoveryKey: _key,
          newPassword: _password,
          deviceName: 'iPhone',
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(state: _signedIn, recoveryKey: sampleRecoveryKey),
        ),
      );
      final bloc = await started();
      bloc.add(submit());
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });

    test('a key that does not match is the library’s words', () async {
      when(
        () => account.recover(
          server: _server,
          username: 'ana',
          recoveryKey: _key,
          newPassword: _password,
          deviceName: 'iPhone',
        ),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, SignedUp>(
          ApplicationError.auth(
            message: 'That recovery key doesn’t match this account.',
          ),
        ),
      );
      final bloc = await started();
      bloc.add(submit());
      await bloc.stream.firstWhere((state) => state.error != null);
      expect(bloc.state.busy, isFalse);
      expect(bloc.state.outcome, isNull);
      await bloc.close();
    });

    test('over notes of another account the question is asked, and the new '
        'key is kept even if it is cancelled', () async {
      when(
        () => account.recover(
          server: _server,
          username: 'ana',
          recoveryKey: _key,
          newPassword: _password,
          deviceName: 'iPhone',
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(
            state: AccountState.question(
              BindingQuestion(
                title: 'This device already has notes',
                body: '3 notes on this device came from another account.',
                answers: <BindingAnswer>[
                  BindingAnswer.add,
                  BindingAnswer.replace,
                ],
              ),
            ),
            recoveryKey: sampleRecoveryKey,
          ),
        ),
      );
      when(() => account.answerBinding(null)).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(
          AccountState.signedOut(),
        ),
      );
      final bloc = await started();
      bloc.add(submit());
      await bloc.stream.firstWhere((state) => state.question != null);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      expect(bloc.state.outcome, isNull);

      bloc.add(const RecoverEvent.bindingAnswered(null));
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedOut);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      await bloc.close();
    });

    test('an answer of Add ends the recovery signed in', () async {
      when(
        () => account.recover(
          server: _server,
          username: 'ana',
          recoveryKey: _key,
          newPassword: _password,
          deviceName: 'iPhone',
        ),
      ).thenAnswer(
        (_) async => const Right<ApplicationError, SignedUp>(
          SignedUp(
            state: AccountState.question(
              BindingQuestion(
                title: 'This device already has notes',
                body: '3 notes on this device came from another account.',
                answers: <BindingAnswer>[BindingAnswer.add],
              ),
            ),
            recoveryKey: sampleRecoveryKey,
          ),
        ),
      );
      when(() => account.answerBinding(BindingAnswer.add)).thenAnswer(
        (_) async => const Right<ApplicationError, AccountState>(_signedIn),
      );
      final bloc = await started();
      bloc.add(submit());
      await bloc.stream.firstWhere((state) => state.question != null);
      bloc.add(const RecoverEvent.bindingAnswered(BindingAnswer.add));
      await bloc.stream.firstWhere((state) => state.outcome != null);
      expect(bloc.state.outcome, AccountOutcome.signedIn);
      expect(bloc.state.recoveryKey, sampleRecoveryKey);
      await bloc.close();
    });
  });
}
