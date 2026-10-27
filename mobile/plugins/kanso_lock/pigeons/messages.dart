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
    kotlinOut: 'android/src/main/kotlin/app/kanso/kanso_lock/Messages.g.kt',
    kotlinOptions: KotlinOptions(package: 'app.kanso.kanso_lock'),
    swiftOut: 'ios/kanso_lock/Sources/kanso_lock/Messages.g.swift',
  ),
)
/// What the phone can do for biometric unlock.
enum BiometricKind {
  /// No biometric hardware, or the platform can't use it.
  none,

  /// Hardware is there but nothing is enrolled.
  notEnrolled,

  /// Face ID (iOS).
  faceId,

  /// Touch ID (iOS).
  touchId,

  /// Optic ID (iOS).
  opticId,

  /// A fingerprint sensor (Android).
  fingerprint,

  /// Face recognition (Android).
  face,

  /// Iris recognition (Android).
  iris,

  /// Strong biometrics of a kind the platform doesn't name (Android).
  biometric,
}

/// How a biometric check ended, as far as Dart is told. The secret itself
/// never comes here: the plugin hands it to Rust.
enum BiometricOutcome {
  /// The check passed and Rust took the secret.
  done,

  /// The person dismissed the prompt or chose the password.
  cancelled,

  /// The enrolled biometrics changed or were removed; the plugin's key is
  /// gone and Rust forgot the kept keys.
  invalidated,

  /// Biometrics can't be used now (none enrolled, locked out, no passcode).
  unavailable,

  /// Anything else.
  failed,
}

@HostApi()
abstract class BiometricsHostApi {
  /// What biometric unlock the phone offers (strong biometrics only).
  BiometricKind availability();

  /// Makes the biometric secret for [libraryId] after a check, and passes it
  /// to Rust for [ticket].
  @async
  BiometricOutcome enroll(int ticket, String libraryId, String reason);

  /// Reads the biometric secret for [libraryId] after a check, and passes it
  /// to Rust for [ticket].
  @async
  BiometricOutcome unlock(int ticket, String libraryId, String reason);

  /// Deletes the platform's key or item for [libraryId].
  void forget(String libraryId);
}

@HostApi()
abstract class PrivacyHostApi {
  /// Keeps the app's contents out of screenshots and the app switcher while
  /// [on] (the cover on iOS, `FLAG_SECURE` on Android).
  void setSecure(bool on);
}
