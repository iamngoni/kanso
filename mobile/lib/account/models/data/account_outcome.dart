// Kanso — account_outcome.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How an account flow ended, for the screen to move on.
enum AccountOutcome {
  /// The account is signed in and open on this phone.
  signedIn,

  /// The account is signed in, but this phone has no key for its notes yet.
  locked,

  /// The phone is signed out.
  signedOut,
}
