// Kanso — discovery_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/models/data/application_error.dart';

/// The local discovery and image reading operations of the open library.
abstract interface class DiscoveryRepository {
  String? meaningQuery(String query);
  Future<Either<ApplicationError, bool>> relatedEnabled();
  Future<Either<ApplicationError, RelatedState>> relatedState();
  Stream<RelatedState> relatedStream();
  Stream<List<String>> changes();
  Future<Either<ApplicationError, Unit>> setRelatedNotes({required bool on});
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  });
  Future<Either<ApplicationError, NoteMeaning>> noteMeaning(String id);
  Future<Either<ApplicationError, Unit>> dismissTwin(String id, String otherId);
  Future<Either<ApplicationError, NoteView>> mergeNotes(
    String keepId,
    String otherId,
  );
  Future<Either<ApplicationError, MeaningResult>> searchByMeaning(
    String query, {
    required int k,
  });
  Future<Either<ApplicationError, GraphLayout>> graph(GraphRequest request);
  Future<Either<ApplicationError, int>> ocrWaiting();
  Future<Either<ApplicationError, ImageToRead?>> ocrNext();
  Future<Either<ApplicationError, bool>> ocrKeep({
    required String id,
    required String md5,
    required String text,
    required String engine,
  });
  Future<Either<ApplicationError, Changed>> addTag(String noteId, String tagId);
}
