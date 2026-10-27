// Kanso — push_registration_phase.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How far this phone's registration for pushes has got.
enum PushRegistrationPhase {
  /// Finding out whether push can work here.
  starting,

  /// It can't on this device (no Firebase configuration, no plugin).
  unavailable,

  /// The OS has been asked for a token and has not answered yet.
  waiting,

  /// The latest token has been handed to the library.
  registered,
}
