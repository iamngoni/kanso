// Kanso — schedule_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/schedule_types.dart';
import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The Schedule: the tasks and dated notes of a range of days.
abstract interface class ScheduleRepository {
  /// What falls on the days of [request], by day and time. Locked notes are
  /// left out.
  Future<Either<ApplicationError, List<ScheduleRow>>> schedule(
    ScheduleRequest request,
  );

  /// The date properties the notebooks [books] define (every notebook when
  /// null): what the property filter offers.
  Future<Either<ApplicationError, List<String>>> dateKeys(List<String>? books);

  /// Moves a note's date property [property], or the task on [line] with
  /// the words [text], to [to], keeping its time, zone and repeat. The
  /// result carries the ticket for Undo.
  Future<Either<ApplicationError, CardMoved>> move({
    required String noteId,
    required String text,
    required DayDate to,
    int? line,
    String? property,
  });
}
