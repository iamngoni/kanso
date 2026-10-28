// Kanso — messages.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:pigeon/pigeon.dart';

/// The typed channel to the native side. Regenerate with
/// `dart run pigeon --input pigeons/messages.dart` in this folder; the
/// generated files are committed.
@ConfigurePigeon(
  PigeonOptions(
    dartOut: 'lib/src/messages.g.dart',
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_dictation/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_dictation'),
    swiftOut: 'ios/kanso_dictation/Sources/kanso_dictation/Messages.g.swift',
  ),
)
/// Whether the person has let the app listen.
enum DictationPermission {
  /// Not asked yet: [DictationHostApi.requestPermission] will ask.
  notDetermined,

  /// Refused; only Settings can change that.
  denied,

  /// Barred by the device's rules (parental controls, a managed device).
  restricted,

  /// Allowed.
  granted,
}

/// Why speech cannot be recognized on this device, for this language.
enum DictationUnavailableReason {
  /// The device has no recognizer at all.
  noRecognizer,

  /// The language has no recognizer that works on the device itself; the
  /// plugin never falls back to a server on iOS.
  notOnDevice,

  /// The language is not one the recognizer knows.
  unsupportedLanguage,
}

/// What a recognizer can do now.
class DictationAvailability {
  DictationAvailability({
    required this.available,
    required this.onDevice,
    required this.permission,
    required this.language,
    this.reason,
  });

  /// Whether speech can be recognized at all.
  bool available;

  /// Whether it is recognized on the device (always on iOS when
  /// [available]; on Android where the offline model is installed).
  bool onDevice;
  DictationPermission permission;

  /// The name of the language asked about, in the device's own language:
  /// "English", "shona".
  String language;

  /// Why not, when [available] is false.
  DictationUnavailableReason? reason;
}

/// What a session reports.
enum DictationEventKind {
  /// The microphone is open and the recognizer is listening.
  listening,

  /// The text so far of the phrase being spoken; it replaces the last one.
  partial,

  /// A phrase is finished; its text is final.
  finalResult,

  /// How loud the voice is now, 0 to 1.
  level,

  /// The session failed; see [DictationEvent.error].
  error,

  /// The session is over, whichever way it ended. The last event of a
  /// session.
  ended,
}

enum DictationErrorCode {
  /// The person has not allowed the microphone or speech recognition.
  permissionDenied,

  /// No recognizer can run now.
  notAvailable,

  /// Nothing was heard.
  noSpeech,

  /// The microphone could not be used, or another app took it.
  audio,

  /// A session is running already.
  busy,

  /// The system interrupted: a call, an alarm, Siri.
  interrupted,

  /// Anything else.
  unknown,
}

class DictationEvent {
  DictationEvent({
    required this.kind,
    this.text,
    this.level,
    this.error,
    this.message,
  });

  DictationEventKind kind;

  /// The words, for [DictationEventKind.partial] and
  /// [DictationEventKind.finalResult].
  String? text;

  /// 0 to 1, for [DictationEventKind.level].
  double? level;

  /// For [DictationEventKind.error].
  DictationErrorCode? error;

  /// A diagnostic for a log; never shown to the person.
  String? message;
}

@HostApi()
abstract class DictationHostApi {
  /// What the recognizer for [localeTag] (a BCP 47 tag; the device's
  /// language when null) can do now.
  @async
  DictationAvailability availability(String? localeTag);

  /// Asks for the microphone and speech recognition if they are not asked
  /// yet, and answers where that leaves them.
  @async
  DictationPermission requestPermission();

  /// Starts listening, in [localeTag] (the device's language when null).
  /// Results arrive as [DictationFlutterApi.onEvent]; the last event of the
  /// session is [DictationEventKind.ended]. A session already running is a
  /// [DictationErrorCode.busy] error event.
  @async
  void start(String? localeTag);

  /// Stops listening and lets the recognizer finish what it heard: a final
  /// result may follow, then ended.
  void stop();

  /// Stops at once and drops what the recognizer had not finished.
  void cancel();

  /// Opens the app's page of the system's Settings.
  void openSettings();
}

@FlutterApi()
abstract class DictationFlutterApi {
  void onEvent(DictationEvent event);
}
