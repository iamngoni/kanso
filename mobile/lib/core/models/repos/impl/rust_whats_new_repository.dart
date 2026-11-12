// Kanso — rust_whats_new_repository.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/whats_new_types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/whats_new_repository.dart';

/// [WhatsNewRepository] over the Rust core's library. The video is kept in
/// the app's support folder, under `whats-new`.
@LazySingleton(as: WhatsNewRepository)
class RustWhatsNewRepository implements WhatsNewRepository {
  RustWhatsNewRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, PhoneWhatsNew?>> whatsNew() =>
      attempt(() async {
        final support = await getApplicationSupportDirectory();
        return _library.whatsNew(cacheDir: p.join(support.path, 'whats-new'));
      });

  @override
  Future<Either<ApplicationError, Unit>> markSeen() => attempt(() async {
    await _library.markWhatsNewSeen();
    return unit;
  });
}
