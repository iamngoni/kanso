// Kanso — unlock_page_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/account/state/unlock/unlock_bloc.dart';
import 'package:kanso/account/views/pages/unlock_page.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';
import 'package:kanso/injection.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_account_blocs.dart';

const String _password = 'correct horse battery';

Future<(FixtureAccountRepository, GoRouter)> _pump(
  WidgetTester tester, {
  bool hasPassword = true,
  bool approvalPending = false,
  List<ApprovalWait> waits = const <ApprovalWait>[],
}) async {
  final account = FixtureAccountRepository()..approvalWaits = waits;
  if (getIt.isRegistered<UnlockBloc>()) {
    getIt.unregister<UnlockBloc>();
  }
  getIt.registerFactory<UnlockBloc>(() => UnlockBloc(account));
  addTearDown(() => unawaited(getIt.reset()));
  final locked = SeededAccountBloc(
    AccountBlocState(
      account: AccountState.locked(
        LockedInfo(
          username: 'ana',
          server: 'https://sync.example.org',
          hasPassword: hasPassword,
          approvalPending: approvalPending,
        ),
      ),
    ),
  );
  closeBlocLater(locked);
  final router = await pumpRouted(
    tester,
    BlocProvider<AccountBloc>.value(value: locked, child: const UnlockPage()),
    size: const Size(393, 1200),
  );
  await tester.pumpAndSettle();
  return (account, router);
}

const ApprovalWait _waiting = ApprovalWait(
  phase: ApprovalPhase.waiting(),
  otherDevices: 1,
  offerRecovery: false,
);

void main() {
  group('UnlockPage', () {
    testWidgets('explains why the notes are closed and offers every way in', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Open your notes on this device'), findsOneWidget);
      expect(
        find.text(
          'You’re signed in as ana, but this device can’t read your '
          'notes yet.',
        ),
        findsOneWidget,
      );
      expect(find.text('Password'), findsOneWidget);
      expect(find.text('Unlock'), findsOneWidget);
      expect(find.text('Approve from another device'), findsOneWidget);
      expect(find.text('Use your recovery key'), findsOneWidget);
      expect(find.text('Sign out'), findsOneWidget);
    });

    testWidgets('an account with no password has no password field', (
      tester,
    ) async {
      await _pump(tester, hasPassword: false);
      expect(find.byType(EditableText), findsNothing);
      expect(find.text('Unlock'), findsNothing);
      expect(find.text('Approve from another device'), findsOneWidget);
    });

    testWidgets('a password unlocks, a short one is told at once', (
      tester,
    ) async {
      final (account, _) = await _pump(tester);
      await tester.enterText(find.byType(EditableText), 'short');
      await tester.tap(find.text('Unlock'));
      await tester.pumpAndSettle();
      expect(
        find.text('That username and password don’t match.'),
        findsOneWidget,
      );
      expect(account.calls, isNot(contains('unlockWithPassword')));

      await tester.enterText(find.byType(EditableText), _password);
      await tester.tap(find.text('Unlock'));
      await tester.pumpAndSettle();
      expect(account.calls, contains('unlockWithPassword'));
    });

    testWidgets('asking another device shows the wait, and Cancel leaves it', (
      tester,
    ) async {
      final (account, _) = await _pump(
        tester,
        waits: const <ApprovalWait>[_waiting],
      );
      await tester.tap(find.text('Approve from another device'));
      await tester.pumpAndSettle();
      expect(account.calls, contains('requestApproval'));
      expect(find.text('Approve this device'), findsOneWidget);
      expect(find.text('Waiting for another device…'), findsOneWidget);
      expect(
        find.text(
          'Open Kanso on a device that’s already signed in and '
          'approve this one.',
        ),
        findsOneWidget,
      );

      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(account.calls, contains('cancelApproval'));
      expect(find.text('Open your notes on this device'), findsOneWidget);
    });

    testWidgets('the code to compare is shown large, with what to do', (
      tester,
    ) async {
      await _pump(
        tester,
        waits: const <ApprovalWait>[
          ApprovalWait(
            phase: ApprovalPhase.compare(),
            code: '123 456',
            otherDevices: 2,
            offerRecovery: false,
          ),
        ],
      );
      await tester.tap(find.text('Approve from another device'));
      await tester.pumpAndSettle();
      expect(find.text('123 456'), findsOneWidget);
      expect(
        find.text(
          'On your other device, approve the request only if it '
          'shows this code.',
        ),
        findsOneWidget,
      );
      expect(find.text('Use your recovery key instead'), findsNothing);
    });

    testWidgets('after two minutes, or with no device, the recovery key is '
        'offered instead', (tester) async {
      final (_, router) = await _pump(
        tester,
        waits: const <ApprovalWait>[
          ApprovalWait(
            phase: ApprovalPhase.noDevices(),
            otherDevices: 0,
            offerRecovery: true,
          ),
        ],
      );
      await tester.tap(find.text('Approve from another device'));
      await tester.pumpAndSettle();
      expect(
        find.text(
          'No other device answered. Use your recovery key to open '
          'this device.',
        ),
        findsOneWidget,
      );
      expect(find.text('Try again'), findsOneWidget);
      await tester.tap(find.text('Use your recovery key instead'));
      await tester.pumpAndSettle();
      expect(topOf(router), startsWith('/settings/account/recover'));
    });

    testWidgets('a request made earlier is picked up on opening', (
      tester,
    ) async {
      final (account, _) = await _pump(
        tester,
        approvalPending: true,
        waits: const <ApprovalWait>[_waiting],
      );
      expect(account.calls, contains('requestApproval'));
      expect(find.text('Approve this device'), findsOneWidget);
    });

    testWidgets('the recovery key button opens the recovery form', (
      tester,
    ) async {
      final (_, router) = await _pump(tester);
      await tester.tap(find.text('Use your recovery key'));
      await tester.pumpAndSettle();
      expect(topOf(router), startsWith('/settings/account/recover'));
    });

    testWidgets('Sign out ends the locked sign-in', (tester) async {
      final (account, _) = await _pump(tester);
      await tester.tap(find.text('Sign out'));
      await tester.pumpAndSettle();
      expect(account.calls, contains('cancelLocked'));
    });
  });
}
