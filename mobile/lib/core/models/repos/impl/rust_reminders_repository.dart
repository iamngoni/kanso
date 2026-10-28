// Kanso — rust_reminders_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/reminders_repository.dart';

/// [RemindersRepository] over the Rust core's library.
@LazySingleton(as: RemindersRepository)
class RustRemindersRepository implements RemindersRepository {
  RustRemindersRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, ReminderPlan>> plan({
    required int nowUtcMs,
    required int offsetMs,
    required bool canNotify,
  }) => attempt(
    () => _library.remindersPlan(
      nowUtcMs: nowUtcMs,
      offsetMs: offsetMs,
      canNotify: canNotify,
    ),
  );

  @override
  Future<Either<ApplicationError, TaskPlace?>> answer({
    required String key,
    required ReminderAnswer action,
    required int nowUtcMs,
  }) => attempt(
    () => _library.reminderAnswer(
      key: key,
      answer: action,
      nowUtcMs: nowUtcMs,
    ),
  );

  @override
  Future<Either<ApplicationError, void>> forget({
    required List<String> keys,
    required int nowUtcMs,
  }) => attempt(
    () => _library.remindersForget(keys: keys, nowUtcMs: nowUtcMs),
  );

  @override
  Future<Either<ApplicationError, void>> forgetFuture({
    required int nowUtcMs,
  }) => attempt(
    () => _library.remindersForgetFuture(nowUtcMs: nowUtcMs),
  );
}
