// Kanso — reminder_notifier.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../bridge/api/types.dart';
import '../../data/reminder_permission.dart';
import '../../data/reminder_response.dart';

/// The system's side of reminders: notifications and the permission to show
/// them (the `app.kanso/reminders` channel).
abstract interface class ReminderNotifier {
  Future<ReminderPermission> permission();

  /// Asks the person; whether they allowed it.
  Future<bool> requestPermission();

  /// Replaces every pending reminder with [reminders].
  Future<void> schedule(List<ReminderNote> reminders);

  /// Shows [reminders] now.
  Future<void> show(List<ReminderNote> reminders);

  /// Removes pending and shown reminders.
  Future<void> cancelAll();

  /// The answers given while the app was not running, then forgotten.
  Future<List<ReminderResponse>> takeResponses();

  /// The answers given while the app runs.
  Stream<ReminderResponse> get responses;
}
