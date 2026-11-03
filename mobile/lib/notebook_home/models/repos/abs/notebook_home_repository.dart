// Kanso — notebook_home_repository.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/notebook_home_types.dart';
import '../../../../core/bridge/api/people_types.dart';
import '../../../../core/models/data/application_error.dart';

/// Notebook homepages over the open library: the page of a notebook, the
/// card above its notes, the Home note and the settings. The words are the
/// core's, so both platforms say the same thing.
abstract interface class NotebookHomeRepository {
  Future<Either<ApplicationError, HomeSettings>> settings();

  Future<Either<ApplicationError, HomeSettings>> setSettings(
    HomeSettings settings,
  );

  Future<Either<ApplicationError, HomeWords>> words();

  Future<Either<ApplicationError, HomePanel?>> open(String bookId);

  /// The map is heavy: read it apart from [open].
  Future<Either<ApplicationError, HomeMap>> map(String bookId);

  Future<Either<ApplicationError, HomeCard>> card(String bookId);

  Future<Either<ApplicationError, HomePanel?>> showAll(
    String bookId,
    String key,
  );

  Future<Either<ApplicationError, HomePanel?>> refreshSection(
    String bookId,
    String key,
  );

  Future<Either<ApplicationError, HomePanel?>> refreshAll(String bookId);

  Future<Either<ApplicationError, HomePanel?>> pin(
    String bookId,
    String key, {
    required bool on,
  });

  Future<Either<ApplicationError, HomePanel?>> hide(
    String bookId,
    String key, {
    required bool hidden,
  });

  Future<Either<ApplicationError, HomePanel?>> showHidden(String bookId);

  Future<Either<ApplicationError, HomePanel?>> notRight(
    String bookId,
    String key,
    int index,
  );

  /// Appends the section to the Home note; the note's ID.
  Future<Either<ApplicationError, String>> copyIntoHome(
    String bookId,
    String key,
  );

  Future<Either<ApplicationError, List<HomeCandidate>>> candidates(
    String bookId,
  );

  Future<Either<ApplicationError, Unit>> chooseHome(
    String bookId,
    String noteId,
  );

  Future<Either<ApplicationError, String>> newHome(String bookId);

  Future<Either<ApplicationError, List<HomeSectionSetting>>> sections(
    String bookId,
  );

  Future<Either<ApplicationError, HomePanel?>> customise(
    String bookId,
    List<String> order,
    List<String> hidden,
  );

  Future<Either<ApplicationError, HomePanel?>> resetLayout(String bookId);

  /// Ticks or unticks a task a section lists.
  Future<Either<ApplicationError, Unit>> setTask(
    PageTask task, {
    required bool checked,
  });

  /// After a note moved: whether its Home note mark was removed.
  Future<Either<ApplicationError, bool>> afterMove(String noteId);
}
