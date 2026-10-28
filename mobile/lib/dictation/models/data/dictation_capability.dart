// Kanso — dictation_capability.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'dictation_permission.dart';
import 'dictation_unavailable_reason.dart';

part 'dictation_capability.freezed.dart';

/// What the phone's recognizer can do now.
@freezed
abstract class DictationCapability with _$DictationCapability {
  const factory DictationCapability({
    /// Whether speech can be recognized at all.
    required bool available,

    /// Whether it is recognized on the phone itself. Always so on iOS when
    /// [available]; on Android false where the system's recognizer, asked to
    /// work offline, stands in for one on the device.
    required bool onDevice,
    required DictationPermission permission,

    /// The name of the language asked about, in the phone's language.
    @Default('') String language,

    /// Why not, while [available] is false.
    DictationUnavailableReason? reason,
  }) = _DictationCapability;
}
