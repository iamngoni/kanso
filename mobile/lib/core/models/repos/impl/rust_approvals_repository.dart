// Kanso — rust_approvals_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/approvals.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/approvals_repository.dart';

/// [ApprovalsRepository] over the Rust core's approvals.
@LazySingleton(as: ApprovalsRepository)
class RustApprovalsRepository implements ApprovalsRepository {
  RustApprovalsRepository(this._approvals);

  final KansoApprovals _approvals;

  @override
  Stream<List<ApprovalPrompt>> watch() => cancelableStream(_approvals.watch());

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> refresh() =>
      attempt(_approvals.refresh);

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> approve(String id) =>
      attempt(() => _approvals.approve(id: id));

  @override
  Future<Either<ApplicationError, List<ApprovalPrompt>>> decline(String id) =>
      attempt(() => _approvals.decline(id: id));
}
