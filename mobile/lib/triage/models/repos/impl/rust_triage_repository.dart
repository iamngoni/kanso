// Kanso — rust_triage_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/triage.dart' as bridge;
import '../../../../core/bridge/api/triage_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/triage_repository.dart';

/// [TriageRepository] over the Rust core's library.
@LazySingleton(as: TriageRepository)
class RustTriageRepository implements TriageRepository {
  RustTriageRepository(KansoLibrary library) : _library = library;

  final KansoLibrary _library;
  bridge.KansoTriage get _triage => _library.triage();

  @override
  Future<Either<ApplicationError, TriageStatus>> status() =>
      attempt(_triage.status);

  @override
  Future<Either<ApplicationError, TriageSettings>> settings() =>
      attempt(_triage.settings);

  @override
  Future<Either<ApplicationError, TriageSettings>> setSettings(
    TriageSettings settings,
  ) => attempt(() => _triage.setSettings(input: settings));

  @override
  Future<Either<ApplicationError, List<TriageCard>>> cards({
    required int limit,
  }) => attempt(() => _triage.cards(limit: limit));

  @override
  Future<Either<ApplicationError, List<TriageChip>>> refine(String noteId) =>
      attempt(() => _triage.refine(noteId: noteId));

  @override
  Future<Either<ApplicationError, Unit>> reject(
    String noteId,
    TriageField field,
    String value,
  ) => attempt(() async {
    await _triage.reject(noteId: noteId, field: field, value: value);
    return unit;
  });

  @override
  Future<Either<ApplicationError, Unit>> unreject(
    String noteId,
    TriageField field,
    String value,
  ) => attempt(() async {
    await _triage.unreject(noteId: noteId, field: field, value: value);
    return unit;
  });

  @override
  Future<Either<ApplicationError, TriageChip>> resolve(
    TriageField field,
    String text,
  ) => attempt(() => _triage.resolve(field: field, text: text));

  @override
  Future<Either<ApplicationError, TriageFiled>> plan(
    List<TriageFiling> filings,
  ) => attempt(() => _triage.plan(filings: filings));

  @override
  Future<Either<ApplicationError, TriageFiled>> file(
    List<TriageFiling> filings,
  ) => attempt(() => _triage.file(filings: filings));

  @override
  Future<Either<ApplicationError, Unit>> turnOnRelatedNotes() =>
      attempt(() async {
        await _library.discovery().setRelatedNotes(on_: true);
        return unit;
      });

  @override
  Stream<List<String>> changes() => _library.changes();
}
