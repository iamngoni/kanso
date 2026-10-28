// Kanso — rust_journal_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/journal_repository.dart';

/// [JournalRepository] over the Rust core's library.
@LazySingleton(as: JournalRepository)
class RustJournalRepository implements JournalRepository {
  RustJournalRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, MadeNote>> today(LocalTime now) =>
      attempt(() => _library.journalToday(now: now));

  @override
  Future<Either<ApplicationError, MadeNote?>> step({
    required String? openNote,
    required LocalTime now,
    required bool forward,
  }) => attempt(
    () => _library.journalStep(openNote: openNote, now: now, forward: forward),
  );

  @override
  Future<Either<ApplicationError, MadeNote>> openDay(
    DayDate day,
    LocalTime now,
  ) => attempt(() => _library.journalOpenDay(day: day, now: now));

  @override
  Future<Either<ApplicationError, MonthGrid>> month({
    required int year,
    required int month,
    required int weekStart,
    required DayDate today,
  }) => attempt(
    () => _library.journalMonth(
      year: year,
      month: month,
      weekStart: weekStart,
      today: today,
    ),
  );

  @override
  Future<DayDate?> dailyDate(String noteId) async =>
      KansoLibrary.dailyDateOf(noteId: noteId);
}
