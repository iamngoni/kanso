// Kanso — push_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/models/data/application_error.dart';
import '../../data/push_token.dart';

/// Waking this phone when notes change elsewhere: the token the OS issues
/// and where it is registered. A push carries nothing; it only says "sync".
abstract interface class PushRepository {
  /// Why push can't work on this phone, in words for a log; null when it
  /// can.
  Future<Either<ApplicationError, String?>> unavailableReason();

  /// The tokens the OS issues, each time it does (a new one included).
  Stream<PushToken> tokens();

  /// Asks the OS for a token. It arrives on [tokens].
  Future<Either<ApplicationError, Unit>> register();

  /// Hands [token] to the library, which registers it with the account's
  /// server when signed in and pushes are on.
  Future<Either<ApplicationError, Unit>> setToken(PushToken token);

  /// Brings the server's copy in line with the settings: the retry after a
  /// registration that failed.
  Future<Either<ApplicationError, Unit>> refresh();

  /// Whether the person wants this phone woken. On until turned off.
  Future<Either<ApplicationError, bool>> isEnabled();

  /// Turns waking this phone on or off; off removes the token from the
  /// server.
  Future<Either<ApplicationError, Unit>> setEnabled({required bool enabled});
}
