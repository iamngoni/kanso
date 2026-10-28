// Kanso — dictation_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'dictation_bloc.dart';

@freezed
sealed class DictationEvent with _$DictationEvent {
  /// A screen that dictates opened: say what the phone can do, so that a
  /// phone that cannot is told at once and not at the first tap.
  const factory DictationEvent.checked() = DictationCheckedEvent;

  /// The person turned the microphone on, to dictate in [localeTag] or the
  /// phone's language.
  const factory DictationEvent.started({
    String? localeTag,
    @Default(false) bool showOverlay,
  }) = DictationStartedEvent;

  /// The person turned the microphone off: finish what was heard and keep it.
  const factory DictationEvent.stopped() = DictationStoppedEvent;

  /// The person is done: finish what was heard, and then say so
  /// ([DictationState.finished]).
  const factory DictationEvent.finishRequested() =
      DictationFinishRequestedEvent;

  /// The person gave up: drop what was heard.
  const factory DictationEvent.cancelled() = DictationCancelledEvent;

  /// The person asked for the phone's Settings, to allow dictation.
  const factory DictationEvent.settingsRequested() =
      DictationSettingsRequestedEvent;

  /// The session reported [signal].
  const factory DictationEvent.signalReceived(DictationSignal signal) =
      DictationSignalReceivedEvent;
}
