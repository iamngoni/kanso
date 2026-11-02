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

  /// Up to three past notes about the paragraph around [caret] (a UTF-16
  /// offset in [body]) of note [id], from the text as it is typed.
  /// [closeness] is "closer", "default" or "looser".
  Future<Either<ApplicationError, ResurfaceResult>> resurface({
    required String id,
    required String title,
    required String body,
    required int caret,
    required String closeness,
    required int utcOffsetSecs,
  });

  /// "Not related": [otherId] is not shown beside note [id] again.
  Future<Either<ApplicationError, Unit>> dismissRelated(
    String id,
    String otherId,
  );

  /// For a daily note, notes written on its date in earlier years.
  Future<Either<ApplicationError, List<OnThisDayRow>>> onThisDay(
    String id, {
    required int utcOffsetSecs,
  });
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
