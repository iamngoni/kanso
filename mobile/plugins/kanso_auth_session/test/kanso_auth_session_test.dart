// Kanso — kanso_auth_session_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_auth_session/kanso_auth_session.dart';
import 'package:kanso_auth_session/src/messages.g.dart';

const String _prefix =
    'dev.flutter.pigeon.kanso_auth_session.AuthSessionHostApi';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  final calls = <List<Object?>>[];

  void answer(String method, Object? reply) {
    messenger.setMockDecodedMessageHandler<Object?>(
      BasicMessageChannel<Object?>(
        '$_prefix.$method',
        AuthSessionHostApi.pigeonChannelCodec,
      ),
      (message) async {
        calls.add(<Object?>[method, message]);
        return <Object?>[reply];
      },
    );
  }

  setUp(calls.clear);

  tearDown(() {
    for (final method in <String>['open', 'close']) {
      messenger.setMockDecodedMessageHandler<Object?>(
        BasicMessageChannel<Object?>(
          '$_prefix.$method',
          AuthSessionHostApi.pigeonChannelCodec,
        ),
        null,
      );
    }
  });

  test('open sends the page and scheme, and returns the callback', () async {
    answer('open', 'kanso://auth/apple?flow=f&s=x');
    final ended = await KansoAuthSession.open(
      'https://sync.example.com/v2/auth/apple/start',
      callbackScheme: 'kanso',
    );
    expect(ended, 'kanso://auth/apple?flow=f&s=x');
    expect(calls.single, <Object?>[
      'open',
      <Object?>['https://sync.example.com/v2/auth/apple/start', 'kanso'],
    ]);
  });

  test('a dismissed session completes with null', () async {
    answer('open', null);
    expect(await KansoAuthSession.open('https://example.com/'), isNull);
    expect(calls.single[1], <Object?>['https://example.com/', null]);
  });

  test('close reaches the native side', () async {
    answer('close', null);
    await KansoAuthSession.close();
    expect(calls.single[0], 'close');
  });

  test('anything but http and https is refused before the channel', () {
    for (final url in <String>[
      'kanso://auth/apple',
      'javascript:alert(1)',
      'file:///etc/passwd',
      'intent://x#Intent;end',
      'data:text/html,hi',
      '//example.com/',
      'example.com',
      'https://',
      '',
    ]) {
      expect(
        () => KansoAuthSession.open(url),
        throwsArgumentError,
        reason: url,
      );
    }
    expect(calls, isEmpty);
  });

  test('http and https pages are web URLs, in any case', () {
    expect(KansoAuthSession.isWebUrl('http://localhost:1/x'), isTrue);
    expect(KansoAuthSession.isWebUrl('HTTPS://example.com/a?b=c'), isTrue);
  });
}
