// Kanso — rust_notebook_home_repository.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../../core/bridge/api/library.dart';
import '../../../../core/bridge/api/notebook_home.dart' as bridge;
import '../../../../core/bridge/api/notebook_home_types.dart';
import '../../../../core/bridge/api/people_types.dart';
import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/notebook_home_repository.dart';

/// [NotebookHomeRepository] over the Rust core's library.
@LazySingleton(as: NotebookHomeRepository)
class RustNotebookHomeRepository implements NotebookHomeRepository {
  RustNotebookHomeRepository(KansoLibrary library) : _library = library;

  final KansoLibrary _library;
  bridge.KansoNotebookHome get _home => _library.notebookHome();

  @override
  Future<Either<ApplicationError, HomeSettings>> settings() =>
      attempt(() => _home.settings());

  @override
  Future<Either<ApplicationError, HomeSettings>> setSettings(
    HomeSettings settings,
  ) => attempt(() => _home.setSettings(input: settings));

  @override
  Future<Either<ApplicationError, HomeWords>> words() =>
      attempt(() => _home.words());

  @override
  Future<Either<ApplicationError, HomePanel?>> open(String bookId) =>
      attempt(() => _home.open(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomeMap>> map(String bookId) =>
      attempt(() => _home.map(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomeCard>> card(String bookId) =>
      attempt(() => _home.card(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomePanel?>> showAll(
    String bookId,
    String key,
  ) => attempt(() => _home.showAll(bookId: bookId, key: key));

  @override
  Future<Either<ApplicationError, HomePanel?>> refreshSection(
    String bookId,
    String key,
  ) => attempt(() => _home.refreshSection(bookId: bookId, key: key));

  @override
  Future<Either<ApplicationError, HomePanel?>> refreshAll(String bookId) =>
      attempt(() => _home.refreshAll(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomePanel?>> pin(
    String bookId,
    String key, {
    required bool on,
  }) => attempt(() => _home.pin(bookId: bookId, key: key, on_: on));

  @override
  Future<Either<ApplicationError, HomePanel?>> hide(
    String bookId,
    String key, {
    required bool hidden,
  }) => attempt(() => _home.hide_(bookId: bookId, key: key, hidden: hidden));

  @override
  Future<Either<ApplicationError, HomePanel?>> showHidden(String bookId) =>
      attempt(() => _home.showHidden(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomePanel?>> notRight(
    String bookId,
    String key,
    int index,
  ) => attempt(() => _home.notRight(bookId: bookId, key: key, index: index));

  @override
  Future<Either<ApplicationError, String>> copyIntoHome(
    String bookId,
    String key,
  ) => attempt(() => _home.copyIntoHome(bookId: bookId, key: key));

  @override
  Future<Either<ApplicationError, List<HomeCandidate>>> candidates(
    String bookId,
  ) => attempt(() => _home.candidates(bookId: bookId));

  @override
  Future<Either<ApplicationError, Unit>> chooseHome(
    String bookId,
    String noteId,
  ) => attempt(() async {
    await _home.chooseHome(bookId: bookId, noteId: noteId);
    return unit;
  });

  @override
  Future<Either<ApplicationError, String>> newHome(String bookId) =>
      attempt(() => _home.newHome(bookId: bookId));

  @override
  Future<Either<ApplicationError, List<HomeSectionSetting>>> sections(
    String bookId,
  ) => attempt(() => _home.sections(bookId: bookId));

  @override
  Future<Either<ApplicationError, HomePanel?>> customise(
    String bookId,
    List<String> order,
    List<String> hidden,
  ) => attempt(
    () => _home.customise(bookId: bookId, order: order, hidden: hidden),
  );

  @override
  Future<Either<ApplicationError, HomePanel?>> resetLayout(String bookId) =>
      attempt(() => _home.resetLayout(bookId: bookId));

  @override
  Future<Either<ApplicationError, Unit>> setTask(
    PageTask task, {
    required bool checked,
  }) => attempt(() async {
    await _library.setTask(
      noteId: task.noteId,
      line: task.line,
      text: task.text,
      checked: checked,
    );
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> afterMove(String noteId) =>
      attempt(() => _home.afterMove(noteId: noteId));
}
