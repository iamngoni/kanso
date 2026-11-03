// Kanso — rust_people_repository.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/people.dart' as bridge;
import '../../../../core/bridge/api/people_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/people_repository.dart';

/// [PeopleRepository] over the Rust core's library.
@LazySingleton(as: PeopleRepository)
class RustPeopleRepository implements PeopleRepository {
  RustPeopleRepository(KansoLibrary library) : _library = library;

  final KansoLibrary _library;
  bridge.KansoPeople get _people => _library.people();

  @override
  Future<Either<ApplicationError, PeopleSettings>> settings() =>
      attempt(_people.settings);

  @override
  Future<Either<ApplicationError, PeopleSettings>> setSettings(
    PeopleSettings settings,
  ) => attempt(() => _people.setSettings(input: settings));

  @override
  Future<Either<ApplicationError, PeopleStatus>> status() =>
      attempt(_people.status);

  @override
  Future<Either<ApplicationError, List<PeoplePage>>> pages() =>
      attempt(_people.pages);

  @override
  Future<Either<ApplicationError, int>> detect({required bool force}) =>
      attempt(() => _people.detect(force: force));

  @override
  Future<Either<ApplicationError, List<PeopleSuggestion>>> suggestions() =>
      attempt(_people.suggestions);

  @override
  Future<Either<ApplicationError, int>> detectWithAi() =>
      attempt(_people.detectWithAi);

  @override
  Future<Either<ApplicationError, String>> create(
    PeopleKind kind,
    String name,
    List<String> aliases,
  ) => attempt(() => _people.create(kind: kind, name: name, aliases: aliases));

  @override
  Future<Either<ApplicationError, Unit>> dismiss(
    String key, {
    required bool never,
  }) => attempt(() async {
    await _people.dismiss(key: key, never: never);
    return unit;
  });

  @override
  Future<Either<ApplicationError, List<String>>> dismissed() =>
      attempt(_people.dismissed);

  @override
  Future<Either<ApplicationError, Unit>> suggestAgain(String name) =>
      attempt(() async {
        await _people.suggestAgain(name: name);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> confirmAlias(
    String key,
    String pageId,
  ) => attempt(() async {
    await _people.confirmAlias(key: key, pageId: pageId);
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> addAlias(
    String pageId,
    String alias,
  ) => attempt(() => _people.addAlias(pageId: pageId, alias: alias));

  @override
  Future<Either<ApplicationError, PeopleMerged>> merge(
    String into,
    String from,
  ) => attempt(() => _people.merge(into: into, from: from));

  @override
  Future<Either<ApplicationError, PagePanel?>> page(String id) =>
      attempt(() => _people.page(id: id));

  @override
  Future<Either<ApplicationError, PageSection?>> showAll(
    String id,
    String key,
  ) => attempt(() => _people.showAll(id: id, key: key));

  @override
  Future<Either<ApplicationError, PagePanel?>> refreshSection(
    String id,
    String key,
  ) => attempt(() => _people.refreshSection(id: id, key: key));

  @override
  Future<Either<ApplicationError, PagePanel?>> refreshAll(String id) =>
      attempt(() => _people.refreshAll(id: id));

  @override
  Future<Either<ApplicationError, PagePanel?>> pin(
    String id,
    String key, {
    required bool on,
  }) => attempt(() => _people.pin(id: id, key: key, on_: on));

  @override
  Future<Either<ApplicationError, PagePanel?>> hide(
    String id,
    String key, {
    required bool hidden,
  }) => attempt(() => _people.hide_(id: id, key: key, hidden: hidden));

  @override
  Future<Either<ApplicationError, PagePanel?>> showHidden(String id) =>
      attempt(() => _people.showHidden(id: id));

  @override
  Future<Either<ApplicationError, PagePanel?>> notRight(
    String id,
    String key,
    int index,
  ) => attempt(() => _people.notRight(id: id, key: key, index: index));

  @override
  Future<Either<ApplicationError, bool>> copyIntoNote(String id, String key) =>
      attempt(() => _people.copyIntoNote(id: id, key: key));

  @override
  Future<Either<ApplicationError, String>> sectionMarkdown(
    String id,
    String key,
  ) => attempt(() => _people.sectionMarkdown(id: id, key: key));
}
