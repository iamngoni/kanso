// Kanso — rust_recording_speakers_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/recording_speakers_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/recording_speakers_repository.dart';

/// [RecordingSpeakersRepository] over the Rust core's library.
@LazySingleton(as: RecordingSpeakersRepository)
class RustRecordingSpeakersRepository implements RecordingSpeakersRepository {
  RustRecordingSpeakersRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, List<SpeakerNameView>>> names(String id) =>
      attempt(() => _library.recordingSpeakers().names(id: id));

  @override
  Future<Either<ApplicationError, SpeakersRenamed>> rename(
    String id,
    List<SpeakerChange> changes,
  ) => attempt(
    () => _library.recordingSpeakers().rename(id: id, changes: changes),
  );
}
