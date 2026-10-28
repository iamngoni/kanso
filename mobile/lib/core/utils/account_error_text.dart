// Kanso — account_error_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/application_error.dart';

/// The words for an account task that failed.
extension AccountErrorText on AppLocalizations {
  /// What an account screen says of [error].
  ///
  /// The library words the failures of account tasks itself, in the desktop's
  /// plain sentences ("That username and password don't match."), never with
  /// the server's text or the network's: those are shown as they come. What
  /// the library did not word (a bug) and a session the server ended get the
  /// app's own lines.
  String accountError(ApplicationError error) {
    return switch (error) {
      InvalidError(:final message) ||
      AuthError(:final message) ||
      NetworkError(:final message) ||
      StorageError(:final message) when message.trim().isNotEmpty => message,
      SignedOutError() => accountSignedOutError,
      _ => accountSomethingWrong,
    };
  }
}
