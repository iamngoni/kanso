// Kanso — reminders_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The reminders row of Settings, with the system's side of reminders
/// played by a handler of the method channel: the real permission prompt
/// cannot be driven from a test.
///
/// Run it with `flutter test integration_test/reminders_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  const channel = MethodChannel('app.kanso/reminders');

  testWidgets('switch reminders off and on, and say when they are blocked', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    final calls = <String>[];
    var permission = 'notDetermined';
    tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(channel, (
      call,
    ) async {
      calls.add(call.method);
      switch (call.method) {
        case 'permission':
          return permission;
        case 'requestPermission':
          permission = 'granted';
          return true;
        case 'takeResponses':
          return <Object?>[];
        default:
          return null;
      }
    });
    addTearDown(
      () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        channel,
        null,
      ),
    );
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, inList('Weekly review'));

    await openLink(tester, '/settings');
    await pumpUntil(tester, find.text('Remind me of tasks'));
    await tester.ensureVisible(find.text('Remind me of tasks'));
    await tester.pumpAndSettle();
    expect(find.textContaining('A notification when a task'), findsOneWidget);

    // Off: the reminders are removed.
    calls.clear();
    await tester.tap(find.text('Remind me of tasks'));
    await pumpUntilTrue(
      tester,
      () => calls.contains('cancelAll'),
      reason: 'the reminders removed',
    );
    expect(calls, isNot(contains('schedule')));

    // On: the person is asked once, and the reminders are planned.
    calls.clear();
    await tester.tap(find.text('Remind me of tasks'));
    await pumpUntilTrue(
      tester,
      () => calls.contains('schedule'),
      reason: 'the reminders scheduled',
    );
    expect(calls, contains('requestPermission'));

    // Blocked by the system: the row says how to allow them.
    permission = 'denied';
    await tester.tap(find.text('Remind me of tasks'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Remind me of tasks'));
    await pumpUntil(tester, find.textContaining('Allow them in your device'));

    semantics.dispose();
  });
}
