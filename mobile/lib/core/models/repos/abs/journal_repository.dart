// Kanso — journal_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The journal: the daily notes, found or made by the Rust core (the same
/// rules as the desktop's). Every call that makes a note gives the note to
/// open.
abstract interface class JournalRepository {
  /// Today's note, made from the daily template if there is none.
  Future<Either<ApplicationError, MadeNote>> today(LocalTime now);

  /// The note of the day before or after the daily note [openNote] (today's
  /// when that is not a daily note or none is open); null when there is no
  /// such day to go to.
  Future<Either<ApplicationError, MadeNote?>> step({
    required String? openNote,
    required LocalTime now,
    required bool forward,
  });

  /// The note of [day], made from the daily template if there is none.
  Future<Either<ApplicationError, MadeNote>> openDay(
    DayDate day,
    LocalTime now,
  );

  /// The month the calendar draws, weeks starting on [weekStart] (0 is
  /// Monday), with the days that have a note marked.
  Future<Either<ApplicationError, MonthGrid>> month({
    required int year,
    required int month,
    required int weekStart,
    required DayDate today,
  });

  /// The day of the daily note [noteId], or null when it is not one.
  Future<DayDate?> dailyDate(String noteId);
}
