// Kanso — sign_in_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/bridge/api/types.dart';

/// What the sign-in screen is doing, which is the button that shows it is
/// busy.
enum SignInAction {
  /// Signing in with a username and password.
  password,

  /// Making an account with a username and password.
  create,

  /// Signing in or making an account with a passkey.
  passkey,

  /// Signing in with Apple.
  apple,

  /// Signing in with Google.
  google,

  /// Signing in with GitHub.
  github,

  /// Making the account for a provider's identity.
  username,

  /// Answering the question about the notes on this phone.
  binding;

  /// The action of signing in with [provider].
  static SignInAction ofProvider(ProviderKind provider) {
    return switch (provider) {
      ProviderKind.apple => SignInAction.apple,
      ProviderKind.google => SignInAction.google,
      ProviderKind.github => SignInAction.github,
    };
  }
}
