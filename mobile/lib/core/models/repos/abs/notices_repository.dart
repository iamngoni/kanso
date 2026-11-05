// Kanso — notices_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/notices_types.dart';
import '../../data/application_error.dart';

/// Update notices: what an update changed that this phone may need to fix.
/// The notices are compiled into the Rust core, shared with the desktop, and
/// never fetched.
abstract interface class NoticesRepository {
  /// The card for the top of the notes list. [signedIn] is whether this
  /// phone has an account.
  Future<Either<ApplicationError, PhoneNotices>> notices({
    required bool signedIn,
  });

  /// Hides the notice [id] on this phone for good.
  Future<Either<ApplicationError, Unit>> dismiss(String id);

  /// Hides the notice [id] because the person fixed what it said.
  Future<Either<ApplicationError, Unit>> markFixed(String id);
}
