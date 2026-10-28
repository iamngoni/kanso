// Kanso — biometrics_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where turning biometric unlock on or off is.
enum BiometricsStatus {
  idle,
  working,
  enabled,
  disabled,

  /// The notes are closed, so biometrics can't be turned on yet.
  notesClosed,
  cancelled,
  failed,
}
