// Kanso — dictation_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'dictation_bloc.dart';

@freezed
abstract class DictationState with _$DictationState {
  const factory DictationState({
    @Default(DictationStatus.idle) DictationStatus status,

    /// The phrases the recognizer finished, in the order they were spoken.
    @Default(<String>[]) List<String> segments,

    /// The words of the phrase being spoken, still changing.
    @Default('') String partial,

    /// The finished phrases as the rules write them (spoken punctuation and
    /// commands acted on): what a note would hold, without the words still
    /// being spoken.
    @Default('') String finishedText,

    /// Everything heard as the rules write it, the words still being spoken
    /// included: what shows is what lands.
    @Default('') String text,

    /// How loud the voice is, 0 to 1.
    @Default(0.0) double level,

    /// Why dictation is unavailable, while [status] is unavailable.
    DictationUnavailableReason? reason,

    /// Why the session failed, while [status] is error.
    DictationFailure? failure,

    /// Whether the phone's rules, not the person, keep dictation from
    /// listening, while [status] is denied.
    @Default(false) bool restricted,

    /// Whether the person asked to be done and the session has ended since.
    @Default(false) bool finished,

    /// Whether the app-level pill should present this dictation session.
    /// Voice-note pages render their own transcript and controls.
    @Default(false) bool showOverlay,
  }) = _DictationState;

  const DictationState._();

  /// Whether the microphone is on or coming on.
  bool get active =>
      status == DictationStatus.listening ||
      status == DictationStatus.requesting;

  /// Everything heard, finished and not: what a note would be made of.
  String get transcript => <String>[
    ...segments,
    if (partial.isNotEmpty) partial,
  ].join(' ');
}
