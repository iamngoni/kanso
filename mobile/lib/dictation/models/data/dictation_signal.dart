// Kanso — dictation_signal.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'dictation_failure.dart';

part 'dictation_signal.freezed.dart';

/// What a session of dictation reports, as it happens.
@freezed
sealed class DictationSignal with _$DictationSignal {
  /// The microphone is open.
  const factory DictationSignal.listening() = DictationListeningSignal;

  /// The words so far of the phrase being spoken; each replaces the last.
  const factory DictationSignal.partial(String text) = DictationPartialSignal;

  /// A phrase is finished, and these are its words.
  const factory DictationSignal.finalText(String text) = DictationFinalSignal;

  /// How loud the voice is, 0 to 1.
  const factory DictationSignal.level(double level) = DictationLevelSignal;

  /// The session failed. An ended signal follows.
  const factory DictationSignal.failed(DictationFailure failure) =
      DictationFailedSignal;

  /// The session is over, whichever way it ended: the last signal of one.
  const factory DictationSignal.ended() = DictationEndedSignal;
}
