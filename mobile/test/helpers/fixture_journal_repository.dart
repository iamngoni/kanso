// Kanso — fixture_journal_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/journal_repository.dart';
import 'package:kanso/core/models/repos/abs/templates_repository.dart';

import 'fixture_library_repository.dart';
import 'library_fixtures.dart';

/// The journal and the templates, in memory, for widget and golden tests:
/// a month is laid out here as the library lays it out, the days with a
/// note are a field, and every call is recorded.
class FixtureJournalRepository
    implements JournalRepository, TemplatesRepository {
  /// The days that have a daily note.
  final Set<(int, int, int)> days = <(int, int, int)>{
    (2026, 10, 3),
    (2026, 10, 12),
    (2026, 10, 19),
    (2026, 10, 25),
    (2026, 10, 26),
  };

  List<TemplateRow> templateRows = const <TemplateRow>[
    TemplateRow(id: 'note:tpl-daily', title: 'Daily'),
    TemplateRow(id: 'note:tpl-meeting', title: 'Meeting notes'),
    TemplateRow(id: 'note:tpl-empty', title: ''),
  ];

  String? daily;

  /// Where the notes these calls make are put, so that opening one finds it.
  /// `registerPageBlocs` points this at the test's library.
  FixtureLibraryRepository? library;

  /// When set, every call that can fail fails with it.
  ApplicationError? failure;

  /// When set, `templates` waits for it.
  Completer<void>? templatesGate;

  /// What was asked, in order: `today`, `step:back`, `day:2026-10-12`,
  /// `from:<template>@<book>`, `newTemplate`, `daily:<id>`.
  final List<String> calls = <String>[];

  static String noteIdOf(DayDate day) =>
      'note:daily-${_two(day.year, 4)}-${_two(day.month)}-${_two(day.day)}';

  static String _two(int value, [int width = 2]) =>
      value.toString().padLeft(width, '0');

  Either<ApplicationError, T> _result<T>(T value) {
    final error = failure;
    return error == null
        ? Right<ApplicationError, T>(value)
        : Left<ApplicationError, T>(error);
  }

  MadeNote _made(DayDate day) {
    days.add((day.year, day.month, day.day));
    final id = noteIdOf(day);
    _putNote(id);
    return MadeNote(id: id, titleFirst: false);
  }

  /// Makes the note the library would have made, so that the route that
  /// opens it finds it.
  void _putNote(String id, {String notebookId = MockIds.journal}) {
    library?.notes[id] = NoteView(
      id: id,
      rev: '1-$id',
      title: '',
      body: '',
      lock: const LockState.plain(),
      notebookId: notebookId,
      tags: const <String>[],
      status: 'active',
      pinned: false,
      createdMs: 1700000000000,
      updatedMs: 1700000000000,
    );
  }

  @override
  Future<DayDate?> dailyDate(String noteId) async {
    final match = RegExp(
      r'^note:daily-(\d{4})-(\d{2})-(\d{2})$',
    ).firstMatch(noteId);
    return match == null
        ? null
        : DayDate(
            year: int.parse(match[1]!),
            month: int.parse(match[2]!),
            day: int.parse(match[3]!),
          );
  }

  DayDate _of(LocalTime now) =>
      DayDate(year: now.year, month: now.month, day: now.day);

  @override
  Future<Either<ApplicationError, MadeNote>> today(LocalTime now) async {
    calls.add('today');
    final error = failure;
    return error != null
        ? Left<ApplicationError, MadeNote>(error)
        : Right<ApplicationError, MadeNote>(_made(_of(now)));
  }

  @override
  Future<Either<ApplicationError, MadeNote?>> step({
    required String? openNote,
    required LocalTime now,
    required bool forward,
  }) async {
    calls.add(forward ? 'step:forward' : 'step:back');
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, MadeNote?>(error);
    }
    final from =
        (openNote == null ? null : await dailyDate(openNote)) ?? _of(now);
    final next = DateTime(
      from.year,
      from.month,
      from.day + (forward ? 1 : -1),
    );
    return Right<ApplicationError, MadeNote?>(
      _made(DayDate(year: next.year, month: next.month, day: next.day)),
    );
  }

  @override
  Future<Either<ApplicationError, MadeNote>> openDay(
    DayDate day,
    LocalTime now,
  ) async {
    calls.add('day:${_two(day.year, 4)}-${_two(day.month)}-${_two(day.day)}');
    final error = failure;
    return error != null
        ? Left<ApplicationError, MadeNote>(error)
        : Right<ApplicationError, MadeNote>(_made(day));
  }

  static const List<String> _monthNames = <String>[
    'January',
    'February',
    'March',
    'April',
    'May',
    'June',
    'July',
    'August',
    'September',
    'October',
    'November',
    'December',
  ];
  static const List<String> _weekdays = <String>[
    'Mon',
    'Tue',
    'Wed',
    'Thu',
    'Fri',
    'Sat',
    'Sun',
  ];

  @override
  Future<Either<ApplicationError, MonthGrid>> month({
    required int year,
    required int month,
    required int weekStart,
    required DayDate today,
  }) async {
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, MonthGrid>(error);
    }
    final first = DateTime(year, month);
    // DateTime.weekday: Monday is 1.
    final back = (first.weekday - 1 + 7 - weekStart) % 7;
    final start = DateTime(year, month, 1 - back);
    final weeks = <List<CalendarDay>>[];
    var cursor = start;
    do {
      final week = <CalendarDay>[];
      for (var i = 0; i < 7; i++) {
        final date = DayDate(
          year: cursor.year,
          month: cursor.month,
          day: cursor.day,
        );
        week.add(
          CalendarDay(
            date: date,
            inMonth: cursor.month == month,
            hasNote: days.contains((date.year, date.month, date.day)),
            isToday: date == today,
          ),
        );
        cursor = DateTime(cursor.year, cursor.month, cursor.day + 1);
      }
      weeks.add(week);
    } while (cursor.month == month);
    return Right<ApplicationError, MonthGrid>(
      MonthGrid(
        year: year,
        month: month,
        title: '${_monthNames[month - 1]} $year',
        weekdayLabels: <String>[
          for (var i = 0; i < 7; i++) _weekdays[(weekStart + i) % 7],
        ],
        weeks: weeks,
      ),
    );
  }

  @override
  Future<Either<ApplicationError, List<TemplateRow>>> templates() async {
    await templatesGate?.future;
    return _result(templateRows);
  }

  @override
  Future<Either<ApplicationError, MadeNote>> newFromTemplate({
    required String templateId,
    required String? bookId,
    required LocalTime now,
  }) async {
    calls.add('from:$templateId@${bookId ?? 'inbox'}');
    final error = failure;
    return error != null
        ? Left<ApplicationError, MadeNote>(error)
        : Right<ApplicationError, MadeNote>(
            _madeFromTemplate(templateId: templateId, bookId: bookId),
          );
  }

  MadeNote _madeFromTemplate({
    required String templateId,
    required String? bookId,
  }) {
    final id = 'note:from-$templateId';
    _putNote(id, notebookId: bookId ?? MockIds.inbox);
    return MadeNote(id: id, titleFirst: true);
  }

  @override
  Future<Either<ApplicationError, MadeNote>> newTemplate() async {
    calls.add('newTemplate');
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, MadeNote>(error);
    }
    const id = 'note:new-template';
    _putNote(id, notebookId: MockIds.templates);
    return const Right<ApplicationError, MadeNote>(
      MadeNote(id: id, titleFirst: true),
    );
  }

  @override
  Future<Either<ApplicationError, String?>> dailyTemplate() async =>
      _result(daily);

  @override
  Future<Either<ApplicationError, Unit>> setDailyTemplate(
    String? templateId,
  ) async {
    calls.add('daily:${templateId ?? 'none'}');
    final error = failure;
    if (error != null) {
      return Left<ApplicationError, Unit>(error);
    }
    daily = templateId;
    return const Right<ApplicationError, Unit>(unit);
  }
}
