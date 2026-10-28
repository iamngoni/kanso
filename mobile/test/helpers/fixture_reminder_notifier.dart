// Kanso — fixture_reminder_notifier.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/reminder_permission.dart';
import 'package:kanso/core/models/data/reminder_response.dart';
import 'package:kanso/core/models/repos/abs/reminder_notifier.dart';

/// The system's side of reminders, in memory: a permission a test sets, the
/// answers waiting, and a record of what was shown and scheduled.
class FixtureReminderNotifier implements ReminderNotifier {
  ReminderPermission current = ReminderPermission.notDetermined;

  /// What `requestPermission` turns the permission into.
  bool allow = true;
  Error? cancelFailure;
  Error? showFailure;
  Error? scheduleFailure;
  int requests = 0;
  int cancels = 0;
  final List<List<ReminderNote>> shown = <List<ReminderNote>>[];
  final List<List<ReminderNote>> scheduled = <List<ReminderNote>>[];
  List<ReminderResponse> waiting = <ReminderResponse>[];
  final StreamController<ReminderResponse> live =
      StreamController<ReminderResponse>.broadcast();

  @override
  Stream<ReminderResponse> get responses => live.stream;

  @override
  Future<ReminderPermission> permission() async => current;

  @override
  Future<bool> requestPermission() async {
    requests++;
    current = allow ? ReminderPermission.granted : ReminderPermission.denied;
    return allow;
  }

  @override
  Future<void> schedule(List<ReminderNote> reminders) async {
    if (scheduleFailure case final failure?) {
      throw failure;
    }
    scheduled.add(reminders);
  }

  @override
  Future<void> show(List<ReminderNote> reminders) async {
    if (showFailure case final failure?) {
      throw failure;
    }
    shown.add(reminders);
  }

  @override
  Future<void> cancelAll() async {
    cancels++;
    if (cancelFailure case final failure?) {
      throw failure;
    }
  }

  @override
  Future<List<ReminderResponse>> takeResponses() async {
    final taken = waiting;
    waiting = <ReminderResponse>[];
    return taken;
  }
}
