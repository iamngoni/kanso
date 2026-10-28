// Kanso — dictation_event_receiver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'messages.g.dart';

/// Takes the events the native side sends and offers them as a stream.
class DictationEventReceiver implements DictationFlutterApi {
  final StreamController<DictationEvent> controller =
      StreamController<DictationEvent>.broadcast();

  @override
  void onEvent(DictationEvent event) => controller.add(event);
}
