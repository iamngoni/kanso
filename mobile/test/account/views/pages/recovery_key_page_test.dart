// Kanso — recovery_key_page_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/state/recovery_key/recovery_key_bloc.dart';
import 'package:kanso/account/views/pages/recovery_key_page.dart';
import 'package:kanso/injection.dart';

import '../../../helpers/fixture_account_repository.dart';
import '../../../helpers/pump_routed.dart';

Future<FixtureAccountRepository> _pump(
  WidgetTester tester, {
  bool withKey = true,
}) async {
  final account = FixtureAccountRepository()..pending = sampleRecoveryKey;
  if (getIt.isRegistered<RecoveryKeyBloc>()) {
    getIt.unregister<RecoveryKeyBloc>();
  }
  getIt.registerFactory<RecoveryKeyBloc>(() => RecoveryKeyBloc(account));
  addTearDown(() => unawaited(getIt.reset()));
  await pumpRouted(
    tester,
    RecoveryKeyPage(recoveryKey: withKey ? sampleRecoveryKey : null),
  );
  await tester.pumpAndSettle();
  return account;
}

void main() {
  group('RecoveryKeyPage', () {
    testWidgets('shows the key large, what it is for and the box', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text(sampleRecoveryKey.key), findsOneWidget);
      expect(find.text('Your recovery key'), findsOneWidget);
      expect(
        find.textContaining('the only way back into your account'),
        findsOneWidget,
      );
      expect(
        find.text('I’ve saved my recovery key somewhere safe'),
        findsOneWidget,
      );
    });

    testWidgets('Done waits for the box, then says the key is saved', (
      tester,
    ) async {
      final account = await _pump(tester);
      // Not pressed yet: nothing happens.
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(account.calls, isNot(contains('recoveryKeySaved')));

      await tester.tap(find.text('I’ve saved my recovery key somewhere safe'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(account.calls, contains('recoveryKeySaved'));
    });

    testWidgets('Copy puts the key on the clipboard', (tester) async {
      await _pump(tester);
      String? copied;
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (call) async {
          if (call.method == 'Clipboard.setData') {
            copied =
                (call.arguments as Map<Object?, Object?>)['text'] as String?;
          }
          return null;
        },
      );
      addTearDown(
        () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
          SystemChannels.platform,
          null,
        ),
      );
      await tester.tap(find.text('Copy recovery key'));
      await tester.pumpAndSettle();
      expect(copied, sampleRecoveryKey.key);
      expect(find.text('Copied'), findsOneWidget);
    });

    testWidgets('without a key given it shows the one the library kept', (
      tester,
    ) async {
      final account = await _pump(tester, withKey: false);
      expect(account.calls, contains('pendingRecoveryKey'));
      expect(find.text(sampleRecoveryKey.key), findsOneWidget);
    });

    testWidgets('with nothing kept there is nothing to save', (tester) async {
      final account = await _pump(tester, withKey: false);
      account.pending = null;
      // A fresh page asks again and finds nothing.
      await pumpRouted(tester, const RecoveryKeyPage());
      await tester.pumpAndSettle();
      expect(find.text('Nothing to save'), findsOneWidget);
    });
  });
}
