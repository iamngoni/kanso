// Kanso — widget_reloader_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/services/widget_reloader.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() {
    messenger.setMockMethodCallHandler(
      const MethodChannel(widgetsChannelName),
      null,
    );
  });

  group('WidgetReloader', () {
    test('is on the channel the platforms answer on', () {
      expect(widgetsChannelName, 'app.kanso.kanso/widgets');
    });

    test('does nothing off a phone', () async {
      // The tests run on a computer: no call goes out.
      final calls = <String>[];
      messenger.setMockMethodCallHandler(
        const MethodChannel(widgetsChannelName),
        (call) async {
          calls.add(call.method);
          return null;
        },
      );
      await WidgetReloader().reload();
      expect(calls, isEmpty);
    });
  });
}
