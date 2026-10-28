// Kanso — approvals_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The devices that ask this phone to let them in: the requests waiting,
/// and the answers.
abstract interface class ApprovalsRepository {
  /// The requests waiting now and each time they change.
  Stream<List<ApprovalPrompt>> watch();

  Future<Either<ApplicationError, List<ApprovalPrompt>>> refresh();

  Future<Either<ApplicationError, List<ApprovalPrompt>>> approve(String id);

  Future<Either<ApplicationError, List<ApprovalPrompt>>> decline(String id);
}
