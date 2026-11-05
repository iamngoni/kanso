// Kanso — rust_recording_summary_repository.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/ai.dart';
import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/recording_summary_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/recording_summary_repository.dart';

/// [RecordingSummaryRepository] over the Rust core's library.
@LazySingleton(as: RecordingSummaryRepository)
class RustRecordingSummaryRepository implements RecordingSummaryRepository {
  RustRecordingSummaryRepository(this._library);

  final KansoLibrary _library;
  KansoAiRun? _run;

  @override
  Future<Either<ApplicationError, SummaryOffer>> offer(String id) =>
      attempt(() => _library.recordingSummary().offer(id: id));

  @override
  Future<Either<ApplicationError, SummaryDraftView>> summarize(String id) =>
      attempt(() {
        final run = _library.ai().newRun();
        _run = run;
        return _library.recordingSummary().summarize(id: id, run: run);
      });

  @override
  void cancel() => _run?.stop();

  @override
  Future<Either<ApplicationError, SummaryApplied>> apply(
    String id,
    SummaryDraftView draft,
    List<int> ticked,
  ) => attempt(
    () => _library.recordingSummary().apply(
      id: id,
      draft: draft,
      ticked: ticked,
    ),
  );
}
