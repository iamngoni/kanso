// Kanso — system calendar bridge tests.
// Created 2026-11-06. Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:kanso/overview/system_calendar.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const MethodChannel channel = MethodChannel(
    'app.kanso.kanso/system_calendar',
  );
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('requests access and lists only native system calendars', () async {
    final calls = <MethodCall>[];
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      return switch (call.method) {
        'requestAccess' => true,
        'calendars' => <Map<String, Object?>>[
          <String, Object?>{
            'id': 'cal-1',
            'name': 'Work',
            'color': '#123456',
            'enabled': true,
          },
        ],
        _ => null,
      };
    });

    final gateway = SystemCalendarGateway();
    expect(await gateway.requestAccess(), isTrue);
    expect(
      await gateway.calendars(),
      const <SystemCalendar>[
        SystemCalendar(
          id: 'cal-1',
          name: 'Work',
          color: '#123456',
          enabled: true,
        ),
      ],
    );
    expect(calls.map((call) => call.method), <String>[
      'requestAccess',
      'calendars',
    ]);
  });

  test('maps events and calendar selections into the engine shape', () async {
    final calls = <MethodCall>[];
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      if (call.method == 'events') {
        return <Map<String, Object?>>[
          <String, Object?>{
            'uid': 'event-1',
            'start': 100,
            'end': 200,
            'title': 'Review',
            'location': 'Room 2',
            'calendarId': 'cal-1',
            'calendar': 'Work',
            'color': '#123456',
            'conferenceUrl': 'https://meet.example.test/one',
            'attendees': <Map<String, Object?>>[
              <String, Object?>{'name': 'Rudo', 'email': 'rudo@example.test'},
            ],
            'allDay': false,
            'free': false,
            'declined': false,
          },
        ];
      }
      return null;
    });

    final gateway = SystemCalendarGateway();
    await gateway.setCalendarEnabled(id: 'cal-1', enabled: false);
    final events = await gateway.events(start: 100, end: 300);
    expect(calls.first.method, 'setCalendarEnabled');
    expect(calls.first.arguments, <String, Object>{
      'id': 'cal-1',
      'enabled': false,
    });
    expect(calls.last.arguments, <String, Object>{'start': 100, 'end': 300});
    expect(events.single.toEngineMap(), <String, Object?>{
      'uid': 'event-1',
      'start': 100,
      'end': 200,
      'title': 'Review',
      'location': 'Room 2',
      'conference_url': 'https://meet.example.test/one',
      'attendees': <Map<String, String>>[
        <String, String>{'name': 'Rudo', 'email': 'rudo@example.test'},
      ],
      'calendar': 'Work',
      'calendar_id': 'cal-1',
      'color': '#123456',
      'all_day': false,
      'free': false,
      'declined': false,
    });
  });

  test('does not call the platform for an empty event range', () async {
    var called = false;
    messenger.setMockMethodCallHandler(channel, (call) async {
      called = true;
      return null;
    });

    expect(await SystemCalendarGateway().events(start: 10, end: 10), isEmpty);
    expect(called, isFalse);
  });
}
