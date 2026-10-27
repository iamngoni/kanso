// Kanso — kanso_passkeys_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_passkeys/kanso_passkeys.dart';
import 'package:kanso_passkeys/src/messages.g.dart';

const BasicMessageChannel<Object?> _channel = BasicMessageChannel<Object?>(
  'dev.flutter.pigeon.kanso_passkeys.PasskeysHostApi.run',
  PasskeysHostApi.pigeonChannelCodec,
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(
    () => messenger.setMockDecodedMessageHandler<Object?>(_channel, null),
  );

  test('run sends the ticket, the kind and the options', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(_channel, (message) async {
      sent = message;
      return <Object?>[PasskeyOutcome.done];
    });
    final outcome = await KansoPasskeys.run(7, 'register', '{"rp_id":"x"}');
    expect(outcome, PasskeyOutcome.done);
    expect(sent, <Object?>[7, 'register', '{"rp_id":"x"}']);
  });

  test('every outcome comes back as it was sent', () async {
    for (final outcome in PasskeyOutcome.values) {
      messenger.setMockDecodedMessageHandler<Object?>(
        _channel,
        (_) async => <Object?>[outcome],
      );
      expect(await KansoPasskeys.run(1, 'assert', '{}'), outcome);
    }
  });

  test('a native error becomes a PlatformException', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      _channel,
      (_) async => <Object?>['failed', 'no activity', null],
    );
    await expectLater(
      KansoPasskeys.run(1, 'assert', '{}'),
      throwsA(isA<PlatformException>()),
    );
  });
}
