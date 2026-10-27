// Kanso — biometric_kinds.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso_lock/kanso_lock.dart';

import '../../gen/l10n/app_localizations.dart';

/// Whether the phone can check biometrics now.
bool biometricsAvailable(BiometricKind kind) =>
    kind != BiometricKind.none && kind != BiometricKind.notEnrolled;

/// What [kind] is called in the app's words: Face ID, Touch ID, fingerprint.
String biometricName(AppLocalizations l10n, BiometricKind kind) {
  return switch (kind) {
    BiometricKind.faceId => l10n.biometricFaceId,
    BiometricKind.touchId => l10n.biometricTouchId,
    BiometricKind.opticId => l10n.biometricOpticId,
    BiometricKind.fingerprint => l10n.biometricFingerprint,
    BiometricKind.face => l10n.biometricFace,
    BiometricKind.iris => l10n.biometricIris,
    BiometricKind.none ||
    BiometricKind.notEnrolled ||
    BiometricKind.biometric => l10n.biometricGeneric,
  };
}
