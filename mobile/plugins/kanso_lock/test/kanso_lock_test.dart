// Kanso — kanso_lock_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_lock/kanso_lock.dart';
import 'package:kanso_lock/src/messages.g.dart';

BasicMessageChannel<Object?> _channel(String api, String method) =>
    BasicMessageChannel<Object?>(
      'dev.flutter.pigeon.kanso_lock.$api.$method',
      BiometricsHostApi.pigeonChannelCodec,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  final enroll = _channel('BiometricsHostApi', 'enroll');
  final unlock = _channel('BiometricsHostApi', 'unlock');
  final availability = _channel('BiometricsHostApi', 'availability');
  final forget = _channel('BiometricsHostApi', 'forget');
  final secure = _channel('PrivacyHostApi', 'setSecure');

  tearDown(() {
    for (final channel in [enroll, unlock, availability, forget, secure]) {
      messenger.setMockDecodedMessageHandler<Object?>(channel, null);
    }
  });

  test('enroll sends the ticket, the library and the reason', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(enroll, (message) async {
      sent = message;
      return <Object?>[BiometricOutcome.done];
    });
    final outcome = await KansoLockPlugin.enroll(7, 'lib-1', 'Turn it on');
    expect(outcome, BiometricOutcome.done);
    expect(sent, <Object?>[7, 'lib-1', 'Turn it on']);
  });

  test('unlock sends the ticket, the library and the reason', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(unlock, (message) async {
      sent = message;
      return <Object?>[BiometricOutcome.cancelled];
    });
    final outcome = await KansoLockPlugin.unlock(9, 'lib-2', 'Unlock');
    expect(outcome, BiometricOutcome.cancelled);
    expect(sent, <Object?>[9, 'lib-2', 'Unlock']);
  });

  test('every outcome comes back as it was sent', () async {
    for (final outcome in BiometricOutcome.values) {
      messenger.setMockDecodedMessageHandler<Object?>(
        unlock,
        (_) async => <Object?>[outcome],
      );
      expect(await KansoLockPlugin.unlock(1, 'l', 'r'), outcome);
    }
  });

  test('every kind comes back as it was sent', () async {
    for (final kind in BiometricKind.values) {
      messenger.setMockDecodedMessageHandler<Object?>(
        availability,
        (_) async => <Object?>[kind],
      );
      expect(await KansoLockPlugin.availability(), kind);
    }
  });

  test('forget and setSecure send their arguments', () async {
    Object? forgot;
    Object? flag;
    messenger
      ..setMockDecodedMessageHandler<Object?>(forget, (message) async {
        forgot = message;
        return <Object?>[null];
      })
      ..setMockDecodedMessageHandler<Object?>(secure, (message) async {
        flag = message;
        return <Object?>[null];
      });
    await KansoLockPlugin.forget('lib-3');
    await KansoLockPlugin.setSecure(on: true);
    expect(forgot, <Object?>['lib-3']);
    expect(flag, <Object?>[true]);
  });

  test('a native error becomes a PlatformException', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      unlock,
      (_) async => <Object?>['failed', 'no activity', null],
    );
    await expectLater(
      KansoLockPlugin.unlock(1, 'l', 'r'),
      throwsA(isA<PlatformException>()),
    );
  });
}
