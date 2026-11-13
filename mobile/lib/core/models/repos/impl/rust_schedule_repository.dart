// Kanso — rust_schedule_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/schedule_types.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/schedule_repository.dart';

/// [ScheduleRepository] over the Rust core's library.
@LazySingleton(as: ScheduleRepository)
class RustScheduleRepository implements ScheduleRepository {
  RustScheduleRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, List<ScheduleRow>>> schedule(
    ScheduleRequest request,
  ) => attempt(() => _library.schedule(request: request));

  @override
  Future<Either<ApplicationError, List<String>>> dateKeys(
    List<String>? books,
  ) => attempt(() => _library.scheduleDateKeys(books: books));

  @override
  Future<Either<ApplicationError, CardMoved>> move({
    required String noteId,
    required String text,
    required DayDate to,
    int? line,
    String? property,
  }) => attempt(
    () => _library.moveScheduled(
      noteId: noteId,
      line: line,
      text: text,
      property: property,
      to: to,
    ),
  );
}
