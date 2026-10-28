// Kanso — kanso_ocr_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso_ocr/kanso_ocr.dart';

BasicMessageChannel<Object?> channel(String method) =>
    BasicMessageChannel<Object?>(
      'dev.flutter.pigeon.kanso_ocr.OcrHostApi.$method',
      OcrHostApi.pigeonChannelCodec,
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;

  tearDown(() {
    debugDefaultTargetPlatformOverride = null;
    for (final method in <String>['read', 'isAvailable']) {
      messenger.setMockDecodedMessageHandler<Object?>(channel(method), null);
    }
  });

  test('is a TextReader', () {
    expect(KansoOcr(), isA<TextReader>());
  });

  test('names Vision on iOS and ML Kit on Android', () {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    expect(KansoOcr().engine, 'vision');
    debugDefaultTargetPlatformOverride = TargetPlatform.android;
    expect(KansoOcr().engine, 'mlkit');
  });

  test('passes the bytes down and returns the lines', () async {
    Object? sent;
    messenger.setMockDecodedMessageHandler<Object?>(channel('read'), (
      message,
    ) async {
      sent = (message! as List<Object?>).single;
      return <Object?>['KANSO\nnotes'];
    });
    final text = await KansoOcr().read(Uint8List.fromList([1, 2, 3]));
    expect(text, 'KANSO\nnotes');
    expect(sent, Uint8List.fromList([1, 2, 3]));
  });

  test('an image with no words reads as an empty string', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      channel('read'),
      (_) async => <Object?>[''],
    );
    expect(await KansoOcr().read(Uint8List(0)), isEmpty);
  });

  test('undecodable bytes throw the unreadable code', () async {
    messenger.setMockDecodedMessageHandler<Object?>(
      channel('read'),
      (_) async => <Object?>['unreadable', 'not an image', null],
    );
    await expectLater(
      KansoOcr().read(Uint8List.fromList([0])),
      throwsA(
        isA<PlatformException>().having(
          (e) => e.code,
          'code',
          KansoOcr.unreadable,
        ),
      ),
    );
  });

  test('availability follows the host and is false when it errors', () async {
    final ocr = KansoOcr();
    messenger.setMockDecodedMessageHandler<Object?>(
      channel('isAvailable'),
      (_) async => <Object?>[true],
    );
    expect(await ocr.available, isTrue);
    messenger.setMockDecodedMessageHandler<Object?>(
      channel('isAvailable'),
      (_) async => <Object?>['boom', null, null],
    );
    expect(await ocr.available, isFalse);
  });
}
