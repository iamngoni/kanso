// Kanso — backups_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'backups_bloc.dart';

@freezed
sealed class BackupsEvent with _$BackupsEvent {
  /// The app opened: make the daily backup if one is due, then read the
  /// list.
  const factory BackupsEvent.started() = BackupsStartedEvent;

  /// The screen was opened, or the list was pulled down: read the list
  /// again.
  const factory BackupsEvent.refreshed() = BackupsRefreshedEvent;

  /// Back up now was pressed.
  const factory BackupsEvent.backUpNow() = BackupsBackUpNowEvent;

  /// The backup made on request at [path] was deleted.
  const factory BackupsEvent.delete(String path) = BackupsDeleteEvent;

  /// The line that says what was just done was dismissed.
  const factory BackupsEvent.noticeCleared() = BackupsNoticeClearedEvent;
}
