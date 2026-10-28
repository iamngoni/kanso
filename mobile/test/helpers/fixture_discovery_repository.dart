// Kanso — fixture_discovery_repository.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/discovery/models/repos/abs/discovery_repository.dart';

/// A disabled local model for browse tests that exercise text search.
class FixtureDiscoveryRepository implements DiscoveryRepository {
  bool relatedOn = false;
  @override
  String? meaningQuery(String query) => null;

  @override
  Future<Either<ApplicationError, RelatedState>> relatedState() async =>
      const Right<ApplicationError, RelatedState>(
        RelatedState(status: RelatedStatus.off()),
      );

  @override
  Stream<RelatedState> relatedStream() => const Stream<RelatedState>.empty();

  @override
  Stream<List<String>> changes() => const Stream<List<String>>.empty();

  @override
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  }) async => const Right<ApplicationError, List<RelatedRow>>(<RelatedRow>[]);

  @override
  Future<Either<ApplicationError, NoteMeaning>> noteMeaning(String id) async =>
      const Right<ApplicationError, NoteMeaning>(
        NoteMeaning(tags: <SuggestedTag>[]),
      );

  @override
  Future<Either<ApplicationError, int>> ocrWaiting() async =>
      const Right<ApplicationError, int>(0);

  @override
  Future<Either<ApplicationError, Unit>> setRelatedNotes({
    required bool on,
  }) async {
    relatedOn = on;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, ImageToRead?>> ocrNext() async =>
      const Right<ApplicationError, ImageToRead?>(null);

  @override
  Future<Either<ApplicationError, bool>> ocrKeep({
    required String id,
    required String md5,
    required String text,
    required String engine,
  }) async => const Right<ApplicationError, bool>(true);

  @override
  Future<Either<ApplicationError, bool>> relatedEnabled() async =>
      Right<ApplicationError, bool>(relatedOn);

  @override
  Future<Either<ApplicationError, MeaningResult>> searchByMeaning(
    String query, {
    required int k,
  }) async => const Right<ApplicationError, MeaningResult>(MeaningResult.off());

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
