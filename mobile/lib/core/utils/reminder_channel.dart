// Kanso — reminder_channel.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart';
import '../models/data/reminder_response.dart';

/// What the native side writes for how a reminder was answered (the
/// desktop's `tasks::reminders::{MARK_DONE, DISMISS}`).
String reminderAnswerWire(ReminderAnswer answer) => switch (answer) {
  ReminderAnswer.open => 'open',
  ReminderAnswer.markDone => 'mark-done',
  ReminderAnswer.dismiss => 'dismiss',
};

/// The answer [wire] names, or null for one this version does not know.
ReminderAnswer? reminderAnswerOf(Object? wire) => switch (wire) {
  'open' => ReminderAnswer.open,
  'mark-done' => ReminderAnswer.markDone,
  'dismiss' => ReminderAnswer.dismiss,
  _ => null,
};

/// The map the `app.kanso/reminders` channel takes for [reminder]: only
/// what the notification shows, the keys to answer it by, and when it
/// fires. Never the note's text.
Map<String, Object> reminderToChannel(ReminderNote reminder) =>
    <String, Object>{
      'key': reminder.key,
      'noteId': reminder.noteId,
      'line': reminder.line,
      'title': reminder.title,
      'body': reminder.body,
      'year': reminder.fireAt.year,
      'month': reminder.fireAt.month,
      'day': reminder.fireAt.day,
      'hour': reminder.fireAt.hour,
      'minute': reminder.fireAt.minute,
    };

/// The response a native map describes; null for one that is not.
ReminderResponse? reminderResponseOf(Object? map) {
  if (map is! Map) {
    return null;
  }
  final key = map['key'];
  final noteId = map['noteId'];
  final line = map['line'];
  final action = reminderAnswerOf(map['action']);
  if (key is! String || noteId is! String || line is! int || action == null) {
    return null;
  }
  return ReminderResponse(
    key: key,
    noteId: noteId,
    line: line,
    action: action,
  );
}
