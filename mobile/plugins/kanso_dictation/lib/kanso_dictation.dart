// Kanso — kanso_dictation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'src/dictation_event_receiver.dart';
import 'src/messages.g.dart';

export 'src/messages.g.dart'
    show
        DictationAvailability,
        DictationErrorCode,
        DictationEvent,
        DictationEventKind,
        DictationPermission,
        DictationUnavailableReason;

/// The platform's speech recognizer, on the device (`SFSpeechRecognizer` on
/// iOS, `SpeechRecognizer` on Android).
///
/// A session is [start] to its last [DictationEventKind.ended] event;
/// [events] carries what it hears. On iOS nothing is ever sent to a server:
/// a language with no recognizer on the device is
/// [DictationUnavailableReason.notOnDevice]. On Android the device's own
/// recognizer is used when it has one, and otherwise the system's with the
/// offline preference; [DictationAvailability.onDevice] says which.
class KansoDictation {
  KansoDictation({DictationHostApi? api})
    : api = api ?? DictationHostApi(),
      receiver = DictationEventReceiver() {
    DictationFlutterApi.setUp(receiver);
  }

  /// The channel to the native side; replaced in tests.
  final DictationHostApi api;
  final DictationEventReceiver receiver;

  /// What a session reports, as it happens.
  Stream<DictationEvent> get events => receiver.controller.stream;

  /// What the recognizer for [localeTag] can do now.
  Future<DictationAvailability> availability([String? localeTag]) =>
      api.availability(localeTag);

  /// Asks for the microphone and speech recognition when not asked yet.
  Future<DictationPermission> requestPermission() => api.requestPermission();

  /// Starts listening in [localeTag], or the device's language.
  Future<void> start([String? localeTag]) => api.start(localeTag);

  /// Stops listening and lets the recognizer finish: a final result may
  /// follow, then the session ends.
  Future<void> stop() => api.stop();

  /// Stops at once, dropping what was not finished.
  Future<void> cancel() => api.cancel();

  /// Opens the app's page of the system's Settings.
  Future<void> openSettings() => api.openSettings();
}
