// Kanso — dictation_permission.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the person has let the app listen: the microphone and speech
/// recognition together.
enum DictationPermission {
  /// Not asked yet.
  notDetermined,

  /// Refused; only the phone's Settings can change that.
  denied,

  /// Barred by the device's rules (parental controls, a managed phone).
  restricted,

  /// Allowed.
  granted,
}
