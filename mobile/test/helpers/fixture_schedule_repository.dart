// Kanso — fixture_schedule_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/schedule_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/schedule_repository.dart';
import 'package:kanso/schedule/utils/schedule_days.dart';

import 'library_fixtures.dart';

/// A row of the Schedule for a fixture, with the fields a test cares about.
ScheduleRow scheduleRow({
  required String noteId,
  required String noteTitle,
  required DayDate date,
  required String title,
  String? bookId,
  ScheduleRowKind kind = ScheduleRowKind.task,
  int? line,
  String? text,
  String? time,
  String? zone,
  String? repeat,
  bool done = false,
  String? property,
  bool projected = false,
}) => ScheduleRow(
  kind: kind,
  noteId: noteId,
  noteTitle: noteTitle,
  bookId: bookId ?? MockIds.kanso,
  line: line,
  text: text ?? title,
  title: title,
  date: date,
  time: time,
  zone: zone,
  repeat: repeat,
  done: done,
  property: property,
  projected: projected,
);

/// The Schedule over rows a test lays out in October 2026 (today is the
/// 26th): the request is answered as the core does, by range, notebook, date
/// property and done, and every move is recorded.
class FixtureScheduleRepository implements ScheduleRepository {
  FixtureScheduleRepository() {
    rows = <ScheduleRow>[
      scheduleRow(
        noteId: 'n-a',
        noteTitle: 'Sync design',
        date: const DayDate(year: 2026, month: 10, day: 26),
        title: 'Send Sam the draft',
        text: 'Send Sam the draft @2026-10-26T15:00',
        line: 4,
        time: '15:00',
      ),
      scheduleRow(
        noteId: 'n-c',
        noteTitle: 'Studio plans',
        date: const DayDate(year: 2026, month: 10, day: 20),
        title: 'Film review',
        text: 'Film review @2026-10-20T15:00 every week',
        line: 7,
        time: '15:00',
        repeat: 'every week',
      ),
      scheduleRow(
        noteId: 'n-c',
        noteTitle: 'Studio plans',
        date: const DayDate(year: 2026, month: 10, day: 27),
        title: 'Film review',
        text: 'Film review @2026-10-20T15:00 every week',
        line: 7,
        time: '15:00',
        repeat: 'every week',
        projected: true,
      ),
      scheduleRow(
        noteId: 'n-d',
        noteTitle: 'Groceries',
        bookId: MockIds.travel,
        date: const DayDate(year: 2026, month: 10, day: 22),
        title: 'Buy oat milk',
        text: 'Buy oat milk @2026-10-22',
        line: 2,
        done: true,
      ),
      scheduleRow(
        noteId: 'n-b',
        noteTitle: 'Pixel 11 review',
        date: const DayDate(year: 2026, month: 10, day: 23),
        title: 'Pixel 11 review',
        kind: ScheduleRowKind.note,
        time: '09:00',
        zone: 'America/New_York',
        property: 'publish',
      ),
      scheduleRow(
        noteId: 'n-e',
        noteTitle: 'Studio tour',
        date: const DayDate(year: 2026, month: 10, day: 26),
        title: 'Studio tour',
        kind: ScheduleRowKind.note,
        property: 'embargo',
      ),
    ];
  }

  /// What `schedule` answers from.
  late List<ScheduleRow> rows;

  /// The date properties `dateKeys` answers.
  List<String> keys = <String>['embargo', 'publish'];

  /// What `schedule` answers instead, when set.
  ApplicationError? scheduleError;

  /// What `move` answers.
  Either<ApplicationError, CardMoved> moveResult =
      const Right<ApplicationError, CardMoved>(
        CardMoved.changed(message: 'Moved to Friday.', undo: 9),
      );

  /// The requests `schedule` was asked.
  final List<ScheduleRequest> requests = <ScheduleRequest>[];

  /// The books `dateKeys` was asked about.
  final List<List<String>?> keyQueries = <List<String>?>[];

  /// The moves asked for: note, line, text, property, day.
  final List<(String, int?, String, String?, DayDate)> moves =
      <(String, int?, String, String?, DayDate)>[];

  @override
  Future<Either<ApplicationError, List<ScheduleRow>>> schedule(
    ScheduleRequest request,
  ) async {
    requests.add(request);
    final error = scheduleError;
    if (error != null) {
      return Left<ApplicationError, List<ScheduleRow>>(error);
    }
    final from = dateOf(request.from);
    final until = dateOf(request.until);
    return Right<ApplicationError, List<ScheduleRow>>(<ScheduleRow>[
      for (final row in rows)
        if (!dateOf(row.date).isBefore(from) &&
            dateOf(row.date).isBefore(until) &&
            (request.books == null || request.books!.contains(row.bookId)) &&
            (row.kind == ScheduleRowKind.task ||
                request.dateKeys == null ||
                request.dateKeys!.contains(row.property)) &&
            (request.includeDone || !row.done))
          row,
    ]);
  }

  @override
  Future<Either<ApplicationError, List<String>>> dateKeys(
    List<String>? books,
  ) async {
    keyQueries.add(books);
    return Right<ApplicationError, List<String>>(keys);
  }

  @override
  Future<Either<ApplicationError, CardMoved>> move({
    required String noteId,
    required String text,
    required DayDate to,
    int? line,
    String? property,
  }) async {
    moves.add((noteId, line, text, property, to));
    return moveResult;
  }
}
