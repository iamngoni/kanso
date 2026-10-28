// Kanso — unlock_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where unlocking a note or notebook is.
enum UnlockStatus {
  idle,
  working,

  /// The locked notes are open now.
  opened,

  /// The password typed was wrong.
  wrongPassword,

  /// The person dismissed the biometric prompt.
  cancelled,

  /// The enrolled biometrics changed; the keys kept for them are gone.
  invalidated,

  /// The phone can't check biometrics now, or they are off.
  unavailable,

  /// The check or the call failed; see the state's error.
  failed,
}
