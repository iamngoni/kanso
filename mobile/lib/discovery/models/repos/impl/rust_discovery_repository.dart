// Kanso — rust_discovery_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/discovery.dart' as bridge;
import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/discovery_repository.dart';

/// The discovery UI adapter over the already generated local Rust bridge.
@LazySingleton(as: DiscoveryRepository)
class RustDiscoveryRepository implements DiscoveryRepository {
  RustDiscoveryRepository(KansoLibrary library) : _library = library;

  final KansoLibrary _library;
  bridge.KansoDiscovery get _discovery => _library.discovery();

  @override
  String? meaningQuery(String query) => bridge.meaningQuery(query: query);

  @override
  Future<Either<ApplicationError, bool>> relatedEnabled() =>
      attempt(_discovery.relatedEnabled);

  @override
  Future<Either<ApplicationError, RelatedState>> relatedState() =>
      attempt(_discovery.relatedState);

  @override
  Stream<RelatedState> relatedStream() => _discovery.relatedStream();

  @override
  Stream<List<String>> changes() => _library.changes();

  @override
  Future<Either<ApplicationError, Unit>> setRelatedNotes({required bool on}) =>
      attempt(() async {
        await _discovery.setRelatedNotes(on_: on);
        return unit;
      });

  @override
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  }) => attempt(() => _discovery.relatedNotes(id: id, limit: limit));

  @override
  Future<Either<ApplicationError, NoteMeaning>> noteMeaning(String id) =>
      attempt(() => _discovery.noteMeaning(id: id));

  @override
  Future<Either<ApplicationError, Unit>> dismissTwin(
    String id,
    String otherId,
  ) => attempt(() async {
    await _discovery.dismissTwin(id: id, otherId: otherId);
    return unit;
  });

  @override
  Future<Either<ApplicationError, NoteView>> mergeNotes(
    String keepId,
    String otherId,
  ) => attempt(() => _discovery.mergeNotes(keepId: keepId, otherId: otherId));

  @override
  Future<Either<ApplicationError, MeaningResult>> searchByMeaning(
    String query, {
    required int k,
  }) => attempt(() => _discovery.searchByMeaning(query: query, k: k));

  @override
  Future<Either<ApplicationError, GraphLayout>> graph(GraphRequest request) =>
      attempt(() => _discovery.graph(request: request));

  @override
  Future<Either<ApplicationError, int>> ocrWaiting() =>
      attempt(_discovery.ocrWaiting);

  @override
  Future<Either<ApplicationError, ImageToRead?>> ocrNext() =>
      attempt(_discovery.ocrNext);

  @override
  Future<Either<ApplicationError, bool>> ocrKeep({
    required String id,
    required String md5,
    required String text,
    required String engine,
  }) => attempt(
    () => _discovery.ocrKeep(id: id, md5: md5, text: text, engine: engine),
  );

  @override
  Future<Either<ApplicationError, Changed>> addTag(
    String noteId,
    String tagId,
  ) => attempt(
    () => _library.change(
      ids: <String>[noteId],
      change: NoteChange.addTag(tagId: tagId),
    ),
  );
}
