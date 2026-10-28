// Kanso — rust_capture_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../services/external_changes.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../../data/library_folder.dart';
import '../abs/capture_repository.dart';

/// [CaptureRepository] over the Rust core's library.
@LazySingleton(as: CaptureRepository)
class RustCaptureRepository implements CaptureRepository {
  RustCaptureRepository(this._library, this._folder, this._external);

  final KansoLibrary _library;
  final LibraryFolder _folder;
  final ExternalChanges _external;

  @override
  Future<Either<ApplicationError, DropsSaved>> saveDrops() => attempt(() async {
    final saved = await _library.saveDrops(inbox: _folder.inboxPath);
    // The share extension saves from its own process: the library's change
    // stream never hears of it, so the lists are told to read again.
    _external.announce();
    return saved;
  });

  @override
  Future<Either<ApplicationError, CaptureOutcome>> capture(Share share) =>
      attempt(() => _library.capture(share: share));

  @override
  Future<Either<ApplicationError, String>> todayNote(DateTime now) => attempt(
    () => _library.todayNote(
      now: LocalTime(
        year: now.year,
        month: now.month,
        day: now.day,
        hour: now.hour,
        minute: now.minute,
        second: now.second,
      ),
    ),
  );
}
