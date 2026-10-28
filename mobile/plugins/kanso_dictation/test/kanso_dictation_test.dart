// Kanso — kanso_dictation_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_dictation/kanso_dictation.dart';
import 'package:kanso_dictation/src/messages.g.dart';

BasicMessageChannel<Object?> channel(String method) =>
    BasicMessageChannel<Object?>(
      'dev.flutter.pigeon.kanso_dictation.DictationHostApi.$method',
      DictationHostApi.pigeonChannelCodec,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() {
    for (final method in <String>[
      'availability',
      'requestPermission',
      'start',
      'stop',
    ]) {
      messenger.setMockDecodedMessageHandler<Object?>(channel(method), null);
    }
  });

  test('availability sends the language and brings back the answer', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(channel('availability'), (
      message,
    ) async {
      sent = message;
      return <Object?>[
        DictationAvailability(
          available: false,
          onDevice: false,
          permission: DictationPermission.denied,
          language: 'Shona',
          reason: DictationUnavailableReason.notOnDevice,
        ),
      ];
    });
    final answer = await KansoDictation().availability('sn-ZW');
    expect(sent, <Object?>['sn-ZW']);
    expect(answer.available, isFalse);
    expect(answer.language, 'Shona');
    expect(answer.permission, DictationPermission.denied);
    expect(answer.reason, DictationUnavailableReason.notOnDevice);
  });

  test('every permission comes back as it was sent', () async {
    final dictation = KansoDictation();
    for (final permission in DictationPermission.values) {
      messenger.setMockDecodedMessageHandler<Object?>(
        channel('requestPermission'),
        (_) async => <Object?>[permission],
      );
      expect(await dictation.requestPermission(), permission);
    }
  });

  test('start sends the language', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(channel('start'), (
      message,
    ) async {
      sent = message;
      return <Object?>[null];
    });
    await KansoDictation().start();
    expect(sent, <Object?>[null]);
  });

  test('a native error becomes a PlatformException', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      channel('stop'),
      (_) async => <Object?>['failed', 'no session', null],
    );
    await expectLater(
      KansoDictation().stop(),
      throwsA(isA<PlatformException>()),
    );
  });

  test('events the native side sends arrive on the stream', () async {
    final dictation = KansoDictation();
    final received = dictation.events.take(2).toList();
    const name =
        'dev.flutter.pigeon.kanso_dictation.DictationFlutterApi.onEvent';
    for (final event in <DictationEvent>[
      DictationEvent(kind: DictationEventKind.partial, text: 'hello'),
      DictationEvent(kind: DictationEventKind.ended),
    ]) {
      await messenger.handlePlatformMessage(
        name,
        DictationFlutterApi.pigeonChannelCodec.encodeMessage(<Object?>[event]),
        (_) {},
      );
    }
    final events = await received;
    expect(events.first.kind, DictationEventKind.partial);
    expect(events.first.text, 'hello');
    expect(events.last.kind, DictationEventKind.ended);
  });
}
