// Kanso — channel_reminder_notifier_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/reminder_permission.dart';
import 'package:kanso/core/models/data/reminder_response.dart';
import 'package:kanso/core/models/repos/impl/channel_reminder_notifier.dart';

const _reminder = ReminderNote(
  key: 'note:a#3',
  noteId: 'note:a',
  line: 3,
  title: 'Call Sam',
  body: '09:00 · Plans',
  fireAt: LocalTime(
    year: 2026,
    month: 10,
    day: 27,
    hour: 9,
    minute: 0,
    second: 0,
  ),
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(remindersChannelName);
  final calls = <MethodCall>[];

  void handle(Future<Object?> Function(MethodCall call) answer) {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) {
          calls.add(call);
          return answer(call);
        });
  }

  setUp(calls.clear);
  tearDown(
    () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null),
  );

  test('uses the channel the native side registers', () {
    expect(remindersChannelName, 'app.kanso/reminders');
  });

  test('asks for the permission and reads it', () async {
    handle((call) async => call.method == 'permission' ? 'denied' : true);
    final notifier = ChannelReminderNotifier();
    expect(await notifier.permission(), ReminderPermission.denied);
    expect(await notifier.requestPermission(), isTrue);
    expect(calls.map((c) => c.method), <String>[
      'permission',
      'requestPermission',
    ]);
  });

  test('an answer it does not know is not determined', () async {
    handle((call) async => 'later');
    expect(
      await ChannelReminderNotifier().permission(),
      ReminderPermission.notDetermined,
    );
  });

  test('schedules and shows maps of the agreed fields only', () async {
    handle((call) async => null);
    final notifier = ChannelReminderNotifier();
    await notifier.schedule(<ReminderNote>[_reminder]);
    await notifier.show(<ReminderNote>[_reminder]);
    await notifier.cancelAll();
    expect(calls.map((c) => c.method), <String>[
      'schedule',
      'show',
      'cancelAll',
    ]);
    final map = (calls.first.arguments as List<Object?>).single! as Map;
    expect(map.keys.toSet(), <String>{
      'key',
      'noteId',
      'line',
      'title',
      'body',
      'year',
      'month',
      'day',
      'hour',
      'minute',
    });
  });

  test('takes the answers given while the app was closed', () async {
    handle(
      (call) async => <Object?>[
        <String, Object>{
          'key': 'k',
          'noteId': 'n',
          'line': 1,
          'action': 'mark-done',
        },
        <String, Object>{'key': 'broken'},
      ],
    );
    final taken = await ChannelReminderNotifier().takeResponses();
    expect(taken, const <ReminderResponse>[
      ReminderResponse(
        key: 'k',
        noteId: 'n',
        line: 1,
        action: ReminderAnswer.markDone,
      ),
    ]);
  });

  test('hears an answer given while the app runs', () async {
    final notifier = ChannelReminderNotifier();
    final heard = <ReminderResponse>[];
    final subscription = notifier.responses.listen(heard.add);
    addTearDown(subscription.cancel);
    await TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .handlePlatformMessage(
          remindersChannelName,
          const StandardMethodCodec().encodeMethodCall(
            const MethodCall('response', <String, Object>{
              'key': 'k',
              'noteId': 'n',
              'line': 2,
              'action': 'open',
            }),
          ),
          (_) {},
        );
    await Future<void>.delayed(Duration.zero);
    expect(heard.single.action, ReminderAnswer.open);
    expect(heard.single.line, 2);
  });

  test(
    'a platform without the channel has no permission and no errors',
    () async {
      final notifier = ChannelReminderNotifier();
      expect(await notifier.permission(), ReminderPermission.notDetermined);
      expect(await notifier.requestPermission(), isFalse);
      await notifier.schedule(<ReminderNote>[_reminder]);
      expect(await notifier.takeResponses(), isEmpty);
    },
  );
}
