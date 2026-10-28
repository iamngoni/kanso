// Kanso — fake_dictation_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso_dictation/kanso_dictation.dart' as plugin;
import 'package:kanso_dictation/src/messages.g.dart' as messages;

/// The native side, as a test: it answers what it is told to and records
/// what it is asked.
class FakeHost extends messages.DictationHostApi {
  plugin.DictationAvailability answer = plugin.DictationAvailability(
    available: true,
    onDevice: true,
    permission: plugin.DictationPermission.granted,
    language: 'English',
  );
  final List<String> calls = <String>[];
  Exception? failure;

  @override
  Future<plugin.DictationAvailability> availability(String? localeTag) async {
    calls.add('availability $localeTag');
    return answer;
  }

  @override
  Future<plugin.DictationPermission> requestPermission() async {
    calls.add('requestPermission');
    return plugin.DictationPermission.granted;
  }

  @override
  Future<void> start(String? localeTag) async {
    calls.add('start $localeTag');
    if (failure != null) {
      throw failure!;
    }
  }

  @override
  Future<void> stop() async => calls.add('stop');

  @override
  Future<void> cancel() async => calls.add('cancel');

  @override
  Future<void> openSettings() async => calls.add('openSettings');
}
