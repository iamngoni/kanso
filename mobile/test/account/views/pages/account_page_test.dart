// Kanso — account_page_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/account/state/account_details/account_details_bloc.dart';
import 'package:kanso/account/views/pages/account_page.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/injection.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/fixture_account_settings_repository.dart';
import '../../../helpers/fixture_auth_session_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_account_blocs.dart';

const String _password = 'correct horse battery';

class _Page {
  _Page(this.settings, this.router);

  final FixtureAccountSettingsRepository settings;
  final GoRouter router;
}

Future<_Page> _pump(
  WidgetTester tester, {
  AccountDetails details = sampleDetails,
}) async {
  final settings = FixtureAccountSettingsRepository(current: details);
  if (getIt.isRegistered<AccountDetailsBloc>()) {
    getIt.unregister<AccountDetailsBloc>();
  }
  getIt.registerFactory<AccountDetailsBloc>(
    () => AccountDetailsBloc(
      settings,
      FixtureAccountRepository(),
      FixtureAuthSessionRepository(),
    ),
  );
  addTearDown(() => unawaited(getIt.reset()));
  final account = SeededAccountBloc(
    const AccountBlocState(
      account: AccountState.signedIn(username: 'ana', server: 's'),
    ),
  );
  closeBlocLater(account);
  final router = await pumpRouted(
    tester,
    BlocProvider<AccountBloc>.value(
      value: account,
      child: const AccountPage(),
    ),
    size: const Size(393, 2000),
  );
  await tester.pumpAndSettle();
  return _Page(settings, router);
}

AccountDetails _without({bool password = true, bool unsaved = false}) =>
    AccountDetails(
      username: sampleDetails.username,
      server: sampleDetails.server,
      devices: sampleDetails.devices,
      storageLine: sampleDetails.storageLine,
      hasPassword: password,
      providers: sampleDetails.providers,
      passkeys: sampleDetails.passkeys,
      recoveryKeyUnsaved: unsaved,
    );

Finder get _fields => find.byType(EditableText);

void main() {
  group('AccountPage', () {
    testWidgets('shows who is signed in, the devices and the ways in', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('ana'), findsOneWidget);
      expect(find.text('sync.kansoapp.app'), findsOneWidget);
      expect(find.text('Ana’s iPhone'), findsOneWidget);
      expect(find.text('This device'), findsOneWidget);
      expect(find.text('Ana’s MacBook'), findsOneWidget);
      expect(find.text('Change'), findsOneWidget);
      expect(find.text('iCloud Keychain'), findsOneWidget);
      expect(find.textContaining('Can unlock a new device'), findsOneWidget);
      expect(find.text('Apple'), findsOneWidget);
      expect(find.text('Unlink'), findsOneWidget);
      expect(find.text('Link'), findsNWidgets(2));
      expect(find.text('Make a new one'), findsOneWidget);
      expect(
        find.text('850 items · 3.1 MB of 100 MB · attachments 14 MB of 500 MB'),
        findsOneWidget,
      );
      expect(find.text('Sign out'), findsNWidgets(2));
      expect(find.text('Delete account'), findsOneWidget);
    });

    testWidgets('another device is signed out only once confirmed', (
      tester,
    ) async {
      final page = await _pump(tester);
      await tester.tap(find.text('Sign out').first);
      await tester.pumpAndSettle();
      expect(find.text('Sign out Ana’s MacBook?'), findsOneWidget);
      expect(
        find.text(
          'It stops syncing until someone signs in on it again. Its notes '
          'stay on it.',
        ),
        findsOneWidget,
      );
      expect(page.settings.calls, isEmpty);

      await tester.tap(find.text('Sign out').last);
      await tester.pumpAndSettle();
      expect(page.settings.calls, <String>['revokeDevice d2']);
      expect(find.text('Signed out Ana’s MacBook.'), findsOneWidget);
    });

    testWidgets('signing this phone out asks, then signs out, and leaves', (
      tester,
    ) async {
      final page = await _pump(tester);
      await tester.ensureVisible(find.text('Sign out').last);
      await tester.tap(find.text('Sign out').last);
      await tester.pumpAndSettle();
      expect(find.text('Sign out of ana?'), findsOneWidget);
      await tester.tap(find.text('Sign out').last);
      await tester.pumpAndSettle();
      expect(page.settings.calls, <String>['signOut']);
    });

    testWidgets('changing the password asks for it and the new one twice, '
        'then shows the new recovery key', (tester) async {
      final page = await _pump(tester);
      await tester.tap(find.text('Change'));
      await tester.pumpAndSettle();
      expect(find.text('Current password'), findsOneWidget);
      expect(find.text('New password'), findsOneWidget);
      expect(find.text('Repeat password'), findsOneWidget);
      await tester.enterText(_fields.at(0), _password);
      await tester.enterText(_fields.at(1), 'another long password');
      await tester.enterText(_fields.at(2), 'another long password');
      await tester.pump();
      await tester.tap(find.text('Change password').last);
      await tester.pumpAndSettle();
      expect(page.settings.calls, <String>['changePassword']);
      expect(topOf(page.router), '/settings/account/recovery-key');
    });

    testWidgets('deleting the account needs the password and its name', (
      tester,
    ) async {
      final page = await _pump(tester);
      await tester.ensureVisible(find.text('Delete account'));
      await tester.tap(find.text('Delete account'));
      await tester.pumpAndSettle();
      expect(find.text('Type ana to confirm'), findsOneWidget);
      await tester.enterText(_fields.at(0), _password);
      await tester.enterText(_fields.at(1), 'ana');
      await tester.pump();
      await tester.tap(find.text('Delete account').last);
      await tester.pumpAndSettle();
      expect(page.settings.calls, <String>['deleteAccount ana']);
    });

    testWidgets('a recovery key not yet saved is offered at the top', (
      tester,
    ) async {
      final page = await _pump(tester, details: _without(unsaved: true));
      expect(
        find.text('Your new recovery key isn’t saved yet'),
        findsOneWidget,
      );
      await tester.tap(find.text('Show it'));
      await tester.pumpAndSettle();
      expect(topOf(page.router), '/settings/account/recovery-key');
    });

    testWidgets('an account with no password cannot change what needs one', (
      tester,
    ) async {
      final page = await _pump(tester, details: _without(password: false));
      expect(find.text('Not set'), findsOneWidget);
      expect(find.text('Change'), findsNothing);
      expect(
        find.textContaining('Confirm it’s you with your password.'),
        findsOneWidget,
      );
      await tester.tap(find.text('Link').first);
      await tester.pumpAndSettle();
      expect(page.settings.calls, isEmpty);
      expect(find.text('Link GitHub'), findsNothing);
    });

    testWidgets('an account that cannot be read says so, and tries again', (
      tester,
    ) async {
      final page = await _pump(tester);
      page.settings.failure = const ApplicationError.network(
        message: 'Can’t reach the server.',
      );
      // A fresh page reads again and fails.
      await pumpRouted(
        tester,
        BlocProvider<AccountBloc>.value(
          value: SeededAccountBloc(
            const AccountBlocState(
              account: AccountState.signedIn(username: 'ana', server: 's'),
            ),
          ),
          child: const AccountPage(),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Your account didn’t load'), findsOneWidget);
      page.settings.failure = null;
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('Ana’s iPhone'), findsOneWidget);
    });
  });
}
