// Kanso — kanso_lock.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'src/messages.g.dart';

export 'src/messages.g.dart' show BiometricKind, BiometricOutcome;

/// Biometric unlock for locked notes and the privacy cover (`LAContext` and
/// the Keychain on iOS, `BiometricPrompt` and the Keystore on Android).
///
/// The plugin passes the biometric secret straight to Rust
/// (`kanso_biometric_answer`); Dart only learns how the check ended. When it
/// did not end well the plugin has also told Rust, so the ticket never waits
/// on a lost answer.
abstract final class KansoLockPlugin {
  static BiometricsHostApi _biometrics = BiometricsHostApi();
  static PrivacyHostApi _privacy = PrivacyHostApi();

  /// Replaces the channels. For tests.
  static void debugApis({
    BiometricsHostApi? biometrics,
    PrivacyHostApi? privacy,
  }) {
    _biometrics = biometrics ?? BiometricsHostApi();
    _privacy = privacy ?? PrivacyHostApi();
  }

  /// What biometric unlock this phone offers (strong biometrics only).
  static Future<BiometricKind> availability() => _biometrics.availability();

  /// Makes the biometric secret of [libraryId] after a check, for [ticket]
  /// (Rust's `begin_biometric_enrollment`). [reason] is the prompt's text.
  static Future<BiometricOutcome> enroll(
    int ticket,
    String libraryId,
    String reason,
  ) => _biometrics.enroll(ticket, libraryId, reason);

  /// Reads the biometric secret of [libraryId] after a check, for [ticket]
  /// (Rust's `begin_biometric_unlock`). [reason] is the prompt's text.
  static Future<BiometricOutcome> unlock(
    int ticket,
    String libraryId,
    String reason,
  ) => _biometrics.unlock(ticket, libraryId, reason);

  /// Deletes the platform's key or item of [libraryId] (after Rust's
  /// `turn_off_biometrics`).
  static Future<void> forget(String libraryId) => _biometrics.forget(libraryId);

  /// Keeps the app's contents out of screenshots and the app switcher while
  /// [on]: the blurred cover on iOS, `FLAG_SECURE` on Android.
  static Future<void> setSecure({required bool on}) => _privacy.setSecure(on);
}
