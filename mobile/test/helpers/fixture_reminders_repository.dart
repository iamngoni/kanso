// Kanso — fixture_reminders_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/reminders_repository.dart';

/// The core's side of reminders for widget tests: an empty plan, and a
/// record of the answers.
class FixtureRemindersRepository implements RemindersRepository {
  final List<(String, ReminderAnswer)> answers = <(String, ReminderAnswer)>[];

  @override
  Future<Either<ApplicationError, ReminderPlan>> plan({
    required int nowUtcMs,
    required int offsetMs,
    required bool canNotify,
  }) async => const Right<ApplicationError, ReminderPlan>(
    ReminderPlan(
      showNow: <ReminderNote>[],
      schedule: <ReminderNote>[],
    ),
  );

  @override
  Future<Either<ApplicationError, TaskPlace?>> answer({
    required String key,
    required ReminderAnswer action,
    required int nowUtcMs,
  }) async {
    answers.add((key, action));
    return const Right<ApplicationError, TaskPlace?>(null);
  }

  @override
  Future<Either<ApplicationError, void>> forget({
    required List<String> keys,
    required int nowUtcMs,
  }) async => const Right<ApplicationError, void>(null);

  @override
  Future<Either<ApplicationError, void>> forgetFuture({
    required int nowUtcMs,
  }) async => const Right<ApplicationError, void>(null);
}
