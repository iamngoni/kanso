// Kanso — platform_dictation_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/dictation_failure.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_signal.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/dictation/models/repos/impl/platform_dictation_repository.dart';
import 'package:kanso_dictation/kanso_dictation.dart' as plugin;

import '../helpers/fake_dictation_host.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late FakeHost host;
  late plugin.KansoDictation dictation;
  late PlatformDictationRepository repository;

  setUp(() {
    host = FakeHost();
    dictation = plugin.KansoDictation(api: host);
    repository = PlatformDictationRepository(dictation);
  });

  test("availability brings the answer in the app's own types", () async {
    host.answer = plugin.DictationAvailability(
      available: false,
      onDevice: false,
      permission: plugin.DictationPermission.restricted,
      language: 'Shona',
      reason: plugin.DictationUnavailableReason.notOnDevice,
    );
    final answer = (await repository.availability(localeTag: 'sn')).toOption();
    final capability = answer.toNullable()!;
    expect(host.calls, <String>['availability sn']);
    expect(capability.available, isFalse);
    expect(capability.permission, DictationPermission.restricted);
    expect(capability.reason, DictationUnavailableReason.notOnDevice);
    expect(capability.language, 'Shona');
  });

  test('start, stop, cancel and openSettings reach the native side', () async {
    await repository.start();
    await repository.stop();
    await repository.cancel();
    await repository.openSettings();
    expect(host.calls, <String>[
      'start null',
      'stop',
      'cancel',
      'openSettings',
    ]);
  });

  test('a failure of the platform is an application error', () async {
    host.failure = PlatformException(code: 'x');
    final result = await repository.start();
    expect(result.isLeft(), isTrue);
    expect(
      result.swap().toOption().toNullable(),
      isA<ApplicationError>(),
    );
  });

  test('events of the platform become signals, in order', () async {
    final signals = repository.signals().take(6).toList();
    for (final event in <plugin.DictationEvent>[
      plugin.DictationEvent(kind: plugin.DictationEventKind.listening),
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.partial,
        text: 'hi',
      ),
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.finalResult,
        text: 'Hi.',
      ),
      plugin.DictationEvent(kind: plugin.DictationEventKind.level, level: 0.5),
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.error,
        error: plugin.DictationErrorCode.noSpeech,
      ),
      plugin.DictationEvent(kind: plugin.DictationEventKind.ended),
    ]) {
      dictation.receiver.onEvent(event);
    }
    expect(await signals, <DictationSignal>[
      const DictationSignal.listening(),
      const DictationSignal.partial('hi'),
      const DictationSignal.finalText('Hi.'),
      const DictationSignal.level(0.5),
      const DictationSignal.failed(DictationFailure.noSpeech),
      const DictationSignal.ended(),
    ]);
  });

  test('an event without its words is dropped', () async {
    final signals = repository.signals().take(1).toList();
    dictation.receiver.onEvent(
      plugin.DictationEvent(kind: plugin.DictationEventKind.partial),
    );
    dictation.receiver.onEvent(
      plugin.DictationEvent(kind: plugin.DictationEventKind.ended),
    );
    expect(await signals, <DictationSignal>[const DictationSignal.ended()]);
  });

  test('every permission and failure maps', () {
    for (final value in plugin.DictationPermission.values) {
      expect(PlatformDictationRepository.permissionOf(value), isNotNull);
    }
    for (final value in plugin.DictationErrorCode.values) {
      expect(PlatformDictationRepository.failureOf(value), isNotNull);
    }
    expect(
      PlatformDictationRepository.failureOf(null),
      DictationFailure.unknown,
    );
    expect(const Right<String, int>(1).isRight(), isTrue);
  });
}
