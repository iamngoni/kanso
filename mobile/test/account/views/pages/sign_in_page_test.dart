// Kanso — sign_in_page_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/account/state/sign_in/sign_in_bloc.dart';
import 'package:kanso/account/views/pages/sign_in_page.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/injection.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/fixture_auth_session_repository.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/pump_routed.dart';

const String _password = 'correct horse battery';

class _Rig {
  _Rig(this.account) {
    if (getIt.isRegistered<SignInBloc>()) {
      getIt.unregister<SignInBloc>();
    }
    getIt.registerFactory<SignInBloc>(
      () => SignInBloc(account, FixtureSyncRepository(), session),
    );
    addTearDown(() => unawaited(getIt.reset()));
  }

  final FixtureAccountRepository account;
  final FixtureAuthSessionRepository session = FixtureAuthSessionRepository();
}

Future<GoRouter> _pump(WidgetTester tester, _Rig rig) async {
  final router = await pumpRouted(tester, const SignInPage());
  await tester.pumpAndSettle();
  return router;
}

Finder get _fields => find.byType(EditableText);

void main() {
  group('SignInPage', () {
    testWidgets('names the server and offers what the server offers', (
      tester,
    ) async {
      await _pump(tester, _Rig(FixtureAccountRepository()));
      expect(find.text('Sign in to Kanso'), findsOneWidget);
      expect(find.text('Kanso sync'), findsOneWidget);
      for (final label in <String>[
        'Continue with Apple',
        'Continue with Google',
        'Continue with GitHub',
        'Sign in with a passkey',
        'Create an account',
      ]) {
        expect(find.text(label), findsOneWidget, reason: label);
      }
      expect(find.textContaining('Forgot your password?'), findsOneWidget);
    });

    testWidgets('leaves out the ways the server does not offer', (
      tester,
    ) async {
      final account = FixtureAccountRepository()
        ..methods = const Right<ApplicationError, ServerMethods>(
          ServerMethods(
            signupsOpen: true,
            password: true,
            passkeys: false,
            providers: <ProviderKind>[ProviderKind.github],
          ),
        );
      await _pump(tester, _Rig(account));
      expect(find.text('Continue with GitHub'), findsOneWidget);
      expect(find.text('Continue with Apple'), findsNothing);
      expect(find.text('Sign in with a passkey'), findsNothing);
    });

    testWidgets('a server with nothing to type in has no form', (tester) async {
      final account = FixtureAccountRepository()
        ..methods = const Right<ApplicationError, ServerMethods>(
          ServerMethods(
            signupsOpen: true,
            password: false,
            passkeys: false,
            providers: <ProviderKind>[ProviderKind.apple],
          ),
        );
      await _pump(tester, _Rig(account));
      expect(_fields, findsNothing);
      expect(find.text('Continue with Apple'), findsOneWidget);
      expect(find.text('or'), findsNothing);
    });

    testWidgets('a server that does not answer says so, and can be asked '
        'again', (tester) async {
      final account = FixtureAccountRepository()
        ..methods = const Left<ApplicationError, ServerMethods>(
          ApplicationError.network(
            message:
                'Can’t reach the server. Your notes are safe on this '
                'device.',
          ),
        );
      await _pump(tester, _Rig(account));
      expect(find.text('This server didn’t answer'), findsOneWidget);
      expect(find.textContaining('Can’t reach the server'), findsOneWidget);
      account.methods = const Right<ApplicationError, ServerMethods>(
        allMethods,
      );
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('Continue with Apple'), findsOneWidget);
    });
  });

  group('SignInPage, signing in', () {
    testWidgets('a form with nothing typed is told what is wrong, and '
        'nothing is sent', (tester) async {
      final rig = _Rig(FixtureAccountRepository());
      await _pump(tester, rig);
      await tester.tap(find.text('Sign in').last);
      await tester.pumpAndSettle();
      expect(find.textContaining('Use 3–32 lowercase letters'), findsOneWidget);
      expect(
        rig.account.calls.where((call) => call.startsWith('signInPassword')),
        isEmpty,
      );
    });

    testWidgets('a password that cannot be right is answered like a wrong '
        'one', (tester) async {
      final rig = _Rig(FixtureAccountRepository());
      await _pump(tester, rig);
      await tester.enterText(_fields.at(0), 'ana');
      await tester.enterText(_fields.at(1), 'short');
      await tester.tap(find.text('Sign in').last);
      await tester.pumpAndSettle();
      expect(
        find.text('That username and password don’t match.'),
        findsOneWidget,
      );
    });

    testWidgets('the button is busy while the account signs in, and the '
        'other ways wait', (tester) async {
      final account = FixtureAccountRepository()..gate = Completer<void>();
      final rig = _Rig(account);
      await _pump(tester, rig);
      await tester.enterText(_fields.at(0), 'ana');
      await tester.enterText(_fields.at(1), _password);
      await tester.tap(find.text('Sign in').last);
      await tester.pump();
      await tester.pump();
      expect(find.text('Signing in…'), findsOneWidget);
      expect(account.calls, contains('signInPassword ana'));

      // A second press while it is busy does nothing.
      await tester.tap(find.text('Signing in…'));
      await tester.pump();
      expect(
        account.calls.where((call) => call == 'signInPassword ana'),
        hasLength(1),
      );

      account.gate!.complete();
      await tester.pumpAndSettle();
      expect(find.text('Signing in…'), findsNothing);
    });

    testWidgets('a refusal is shown in the library’s words', (tester) async {
      final account = FixtureAccountRepository()
        ..signedIn = const Left<ApplicationError, AccountState>(
          ApplicationError.auth(
            message: 'That username and password don’t match.',
          ),
        );
      await _pump(tester, _Rig(account));
      await tester.enterText(_fields.at(0), 'ana');
      await tester.enterText(_fields.at(1), _password);
      await tester.tap(find.text('Sign in').last);
      await tester.pumpAndSettle();
      expect(
        find.text('That username and password don’t match.'),
        findsOneWidget,
      );
      expect(find.text('Sign in').last, findsOneWidget);
    });

    testWidgets('the recovery link opens the recovery form for the server', (
      tester,
    ) async {
      final rig = _Rig(FixtureAccountRepository());
      final router = await _pump(tester, rig);
      await tester.tap(find.textContaining('Forgot your password?'));
      await tester.pumpAndSettle();
      expect(
        topOf(router),
        startsWith('/settings/account/recover'),
      );
    });
  });

  group('SignInPage, the server and an account', () {
    testWidgets('another server is typed, and asked what it offers', (
      tester,
    ) async {
      final rig = _Rig(FixtureAccountRepository());
      await _pump(tester, rig);
      await tester.tap(find.text('Server'));
      await tester.pumpAndSettle();
      expect(find.text('Server address'), findsOneWidget);
      await tester.enterText(_fields.at(0), 'https://sync.example.org');
      await tester.tap(find.text('Use this server'));
      await tester.pumpAndSettle();
      expect(
        rig.account.calls,
        contains('serverMethods https://sync.example.org'),
      );
      expect(find.text('sync.example.org'), findsOneWidget);
    });

    testWidgets('Create an account shows its form, and back returns', (
      tester,
    ) async {
      await _pump(tester, _Rig(FixtureAccountRepository()));
      await tester.tap(find.text('Create an account'));
      await tester.pumpAndSettle();
      expect(find.text('Create your account'), findsOneWidget);
      expect(find.text('Repeat password'), findsOneWidget);
      expect(find.text('Create with a passkey'), findsOneWidget);
      expect(
        find.text(
          'Kanso can’t reset your password; you’ll get a recovery key '
          'instead.',
        ),
        findsOneWidget,
      );

      await tester.tap(find.bySemanticsLabel('Back'));
      await tester.pumpAndSettle();
      expect(find.text('Sign in to Kanso'), findsOneWidget);
    });

    testWidgets('a new account is checked, then made', (tester) async {
      final rig = _Rig(FixtureAccountRepository());
      await _pump(tester, rig);
      await tester.tap(find.text('Create an account'));
      await tester.pumpAndSettle();
      await tester.enterText(_fields.at(0), 'ana');
      await tester.enterText(_fields.at(1), _password);
      await tester.enterText(_fields.at(2), 'a different password');
      await tester.tap(find.text('Create account'));
      await tester.pumpAndSettle();
      expect(find.text('The passwords don’t match.'), findsOneWidget);
      expect(
        rig.account.calls.where((c) => c.startsWith('signUpPassword')),
        isEmpty,
      );

      await tester.enterText(_fields.at(2), _password);
      await tester.tap(find.text('Create account'));
      await tester.pumpAndSettle();
      expect(rig.account.calls, contains('signUpPassword ana'));
    });

    testWidgets('a closed server says it takes no accounts', (tester) async {
      final account = FixtureAccountRepository()
        ..methods = const Right<ApplicationError, ServerMethods>(
          ServerMethods(
            signupsOpen: false,
            password: true,
            passkeys: false,
            providers: <ProviderKind>[],
          ),
        );
      await _pump(tester, _Rig(account));
      await tester.tap(find.text('Create an account'));
      await tester.pumpAndSettle();
      expect(
        find.text('This server isn’t taking new accounts.'),
        findsOneWidget,
      );
    });
  });
}
