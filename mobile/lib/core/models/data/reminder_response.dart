// Kanso — reminder_response.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart' show ReminderAnswer;

part 'reminder_response.freezed.dart';

/// A reminder the person answered on the notification.
@freezed
abstract class ReminderResponse with _$ReminderResponse {
  const factory ReminderResponse({
    required String key,
    required String noteId,
    required int line,
    required ReminderAnswer action,
  }) = _ReminderResponse;
}
