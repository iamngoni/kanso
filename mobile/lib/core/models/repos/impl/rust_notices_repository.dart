// Kanso — rust_notices_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/notices_types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/notices_repository.dart';

/// [NoticesRepository] over the Rust core's library.
@LazySingleton(as: NoticesRepository)
class RustNoticesRepository implements NoticesRepository {
  RustNoticesRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, PhoneNotices>> notices({
    required bool signedIn,
  }) => attempt(() => _library.updateNotices(signedInToSync: signedIn));

  @override
  Future<Either<ApplicationError, Unit>> dismiss(String id) =>
      attempt(() async {
        await _library.dismissUpdateNotice(id: id);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> markFixed(String id) =>
      attempt(() async {
        await _library.markUpdateNoticeFixed(id: id);
        return unit;
      });
}
