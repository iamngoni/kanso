// Kanso — whats_new_repository.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/whats_new_types.dart';
import '../../data/application_error.dart';

/// What's new, once after an update: the release's notes and, when it has
/// one, its short video. The Rust core reads the release's signed feed,
/// checks the video against it and keeps it on the phone.
abstract interface class WhatsNewRepository {
  /// The sheet to show now, or null when there is nothing new to show.
  Future<Either<ApplicationError, PhoneWhatsNew?>> whatsNew();

  /// Notes that this version's sheet was shown.
  Future<Either<ApplicationError, Unit>> markSeen();
}
