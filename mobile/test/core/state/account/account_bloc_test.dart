// Kanso — account_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const AccountState _signedIn = AccountState.signedIn(
  username: 'ana',
  server: 'https://sync.example.invalid',
);
const LockedInfo _info = LockedInfo(
  username: 'ana',
  server: 'https://sync.example.invalid',
  hasPassword: true,
  approvalPending: false,
);

void main() {
  late MockAccountRepository account;

  setUp(() => account = MockAccountRepository());

  group('AccountBloc', () {
    test('knows nothing until the account is first read', () {
      final bloc = AccountBloc(account);
      expect(bloc.state.account, isNull);
      expect(bloc.state.signedIn, isFalse);
      expect(bloc.state.locked, isNull);
      expect(bloc.state.username, isNull);
      unawaited(bloc.close());
    });

    blocTest<AccountBloc, AccountBlocState>(
      'follows the account as it changes',
      build: () {
        when(account.stateStream).thenAnswer(
          (_) => Stream<AccountState>.fromIterable(const <AccountState>[
            AccountState.signedOut(),
            AccountState.locked(_info),
            _signedIn,
          ]),
        );
        return AccountBloc(account);
      },
      act: (bloc) => bloc.add(const AccountEvent.started()),
      expect: () => const <AccountBlocState>[
        AccountBlocState(account: AccountState.signedOut()),
        AccountBlocState(account: AccountState.locked(_info)),
        AccountBlocState(account: _signedIn),
      ],
    );

    test('says who is signed in, locked or not', () {
      expect(const AccountBlocState(account: _signedIn).signedIn, isTrue);
      expect(const AccountBlocState(account: _signedIn).username, 'ana');
      const locked = AccountBlocState(account: AccountState.locked(_info));
      expect(locked.signedIn, isFalse);
      expect(locked.locked, _info);
      expect(locked.username, 'ana');
      const asking = AccountBlocState(
        account: AccountState.question(
          BindingQuestion(
            title: 'This device already has notes',
            body: 'They are not this account’s.',
            answers: <BindingAnswer>[BindingAnswer.add],
          ),
        ),
      );
      expect(asking.signedIn, isFalse);
      expect(asking.locked, isNull);
      expect(asking.username, isNull);
    });

    test('a changing stream is followed until the bloc is closed', () async {
      final changes = StreamController<AccountState>();
      when(account.stateStream).thenAnswer((_) => changes.stream);
      final bloc = AccountBloc(account)..add(const AccountEvent.started());
      changes.add(_signedIn);
      await bloc.stream.firstWhere((state) => state.signedIn);
      changes.add(const AccountState.signedOut());
      await bloc.stream.firstWhere((state) => !state.signedIn);
      await bloc.close();
      unawaited(changes.close());
    });
  });
}
