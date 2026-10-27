// Kanso — kanso_background_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_background/kanso_background.dart';
import 'package:kanso_background/src/messages.g.dart';

const String _prefix = 'dev.flutter.pigeon.kanso_background.BackgroundHostApi';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  final calls = <String>[];

  void answer(String method, Object? Function() reply) {
    messenger.setMockDecodedMessageHandler<Object?>(
      BasicMessageChannel<Object?>(
        '$_prefix.$method',
        BackgroundHostApi.pigeonChannelCodec,
      ),
      (_) async {
        calls.add(method);
        return <Object?>[reply()];
      },
    );
  }

  setUp(calls.clear);

  tearDown(() {
    for (final method in <String>['runNow', 'schedule']) {
      messenger.setMockDecodedMessageHandler<Object?>(
        BasicMessageChannel<Object?>(
          '$_prefix.$method',
          BackgroundHostApi.pigeonChannelCodec,
        ),
        null,
      );
    }
  });

  test('runNow returns the native code', () async {
    answer('runNow', () => 1);
    expect(await KansoBackground.runNow(), 1);
    expect(calls, <String>['runNow']);
  });

  test('every documented code passes through unchanged', () async {
    for (final code in <int>[1, 0, -1, -2, -3, -4, -5]) {
      answer('runNow', () => code);
      expect(await KansoBackground.runNow(), code);
    }
  });

  test('schedule reaches the native side', () async {
    answer('schedule', () => null);
    await KansoBackground.schedule();
    expect(calls, <String>['schedule']);
  });

  test('a native error becomes a PlatformException', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      const BasicMessageChannel<Object?>(
        '$_prefix.runNow',
        BackgroundHostApi.pigeonChannelCodec,
      ),
      (_) async => <Object?>['failed', 'no library', null],
    );
    await expectLater(
      KansoBackground.runNow(),
      throwsA(isA<PlatformException>()),
    );
  });
}
