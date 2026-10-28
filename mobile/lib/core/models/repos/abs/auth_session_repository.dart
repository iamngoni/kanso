// Kanso — auth_session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';

/// The system's sign-in session for the pages of Apple, Google and GitHub.
abstract interface class AuthSessionRepository {
  /// Opens [url]. Completes with the address the session ended at when it
  /// ended at [callbackScheme], and with null when it was dismissed or
  /// [close] ended it.
  Future<Either<ApplicationError, String?>> open(
    String url, {
    String? callbackScheme,
  });

  /// Ends the open session, if there is one.
  Future<Either<ApplicationError, Unit>> close();
}
