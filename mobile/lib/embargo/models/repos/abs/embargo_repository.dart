// Kanso — embargo_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/embargo_types.dart';
import '../../../../core/models/data/application_error.dart';

/// A note's embargo: whether its `embargo` property is still to come, for
/// the banner under its title and the question before publishing it, making
/// a share link to it or exporting it. The rules are the Rust core's.
// ignore: one_member_abstracts
abstract interface class EmbargoRepository {
  /// The embargo of note [noteId] now, or null when it has none, it has
  /// lifted or the note is locked.
  Future<Either<ApplicationError, EmbargoView?>> embargo(String noteId);
}
