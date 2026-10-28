// Kanso — reminders_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The Rust core's side of reminders: which to show and schedule, and what
/// an answer does. The system's side is a `ReminderNotifier`.
abstract interface class RemindersRepository {
  /// What to show now and to schedule. [canNotify] false (reminders are off
  /// or the system said no) gives an empty plan and records nothing.
  /// [offsetMs] is the device's offset from UTC.
  Future<Either<ApplicationError, ReminderPlan>> plan({
    required int nowUtcMs,
    required int offsetMs,
    required bool canNotify,
  });

  /// Applies the answer to the reminder [key]: Mark Done ticks the task.
  /// The task's place, for the answer that opens it.
  Future<Either<ApplicationError, TaskPlace?>> answer({
    required String key,
    required ReminderAnswer action,
    required int nowUtcMs,
  });

  /// Removes only plan entries whose platform delivery failed.
  Future<Either<ApplicationError, void>> forget({
    required List<String> keys,
    required int nowUtcMs,
  });

  /// Clears future schedule records after reminders are disabled or denied.
  Future<Either<ApplicationError, void>> forgetFuture({
    required int nowUtcMs,
  });
}
