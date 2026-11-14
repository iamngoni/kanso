// Kanso — platform_capture_window_repository_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/capture/models/data/capture_mode.dart';
import 'package:kanso/capture/models/repos/impl/platform_capture_window_repository.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel(PlatformCaptureWindowRepository.channelName);
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  late List<MethodCall> calls;

  void answer(Object? Function(MethodCall call) reply) {
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      return reply(call);
    });
  }

  setUp(() => calls = <MethodCall>[]);
  tearDown(() => messenger.setMockMethodCallHandler(channel, null));

  test('the channel is the one CaptureActivity answers', () {
    expect(PlatformCaptureWindowRepository.channelName, 'app.kanso/capture');
  });

  test('mode reads task and note', () async {
    final repository = PlatformCaptureWindowRepository();
    answer((_) => 'task');
    expect(await repository.mode(), CaptureMode.task);
    answer((_) => 'note');
    expect(await repository.mode(), CaptureMode.note);
    expect(calls.map((call) => call.method), <String>['mode', 'mode']);
  });

  test('mode is Note when the window cannot say', () async {
    final repository = PlatformCaptureWindowRepository();
    answer((_) => throw PlatformException(code: 'gone'));
    expect(await repository.mode(), CaptureMode.note);
    messenger.setMockMethodCallHandler(channel, null);
    expect(await repository.mode(), CaptureMode.note);
  });

  test('finish and openInKanso ask the window', () async {
    final repository = PlatformCaptureWindowRepository();
    answer((_) => null);
    await repository.openInKanso('kanso://open/new');
    await repository.finish();
    expect(calls.map((call) => call.method), <String>['open', 'finish']);
    expect(calls.first.arguments, 'kanso://open/new');
  });

  test('a window that fails or is missing is not an error', () async {
    final repository = PlatformCaptureWindowRepository();
    answer((_) => throw PlatformException(code: 'gone'));
    await expectLater(repository.finish(), completes);
    messenger.setMockMethodCallHandler(channel, null);
    await expectLater(repository.openInKanso('kanso://open/new'), completes);
  });
}
