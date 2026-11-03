// Kanso — people_repository.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/people_types.dart';
import '../../../../core/models/data/application_error.dart';

/// People & Projects over the open library: the pages, the suggestions, a
/// page's live sections and the settings. Nothing is made or merged unless a
/// caller asks.
abstract interface class PeopleRepository {
  Future<Either<ApplicationError, PeopleSettings>> settings();

  Future<Either<ApplicationError, PeopleSettings>> setSettings(
    PeopleSettings settings,
  );

  Future<Either<ApplicationError, PeopleStatus>> status();

  Future<Either<ApplicationError, List<PeoplePage>>> pages();

  Future<Either<ApplicationError, int>> detect({required bool force});

  Future<Either<ApplicationError, List<PeopleSuggestion>>> suggestions();

  Future<Either<ApplicationError, int>> detectWithAi();

  Future<Either<ApplicationError, String>> create(
    PeopleKind kind,
    String name,
    List<String> aliases,
  );

  Future<Either<ApplicationError, Unit>> dismiss(
    String key, {
    required bool never,
  });

  Future<Either<ApplicationError, List<String>>> dismissed();

  Future<Either<ApplicationError, Unit>> suggestAgain(String name);

  Future<Either<ApplicationError, Unit>> confirmAlias(
    String key,
    String pageId,
  );

  Future<Either<ApplicationError, bool>> addAlias(String pageId, String alias);

  Future<Either<ApplicationError, PeopleMerged>> merge(
    String into,
    String from,
  );

  Future<Either<ApplicationError, PagePanel?>> page(String id);

  Future<Either<ApplicationError, PageSection?>> showAll(String id, String key);

  Future<Either<ApplicationError, PagePanel?>> refreshSection(
    String id,
    String key,
  );

  Future<Either<ApplicationError, PagePanel?>> refreshAll(String id);

  Future<Either<ApplicationError, PagePanel?>> pin(
    String id,
    String key, {
    required bool on,
  });

  Future<Either<ApplicationError, PagePanel?>> hide(
    String id,
    String key, {
    required bool hidden,
  });

  Future<Either<ApplicationError, PagePanel?>> showHidden(String id);

  Future<Either<ApplicationError, PagePanel?>> notRight(
    String id,
    String key,
    int index,
  );

  Future<Either<ApplicationError, bool>> copyIntoNote(String id, String key);

  Future<Either<ApplicationError, String>> sectionMarkdown(
    String id,
    String key,
  );
}
