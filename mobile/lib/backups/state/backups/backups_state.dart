// Kanso — backups_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'backups_bloc.dart';

@freezed
abstract class BackupsState with _$BackupsState {
  const factory BackupsState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The backups, newest first.
    @Default(<BackupFile>[]) List<BackupFile> backups,

    /// A backup is being made on request.
    @Default(false) bool working,

    /// The backups being deleted now, by path.
    @Default(<String>{}) Set<String> deleting,

    /// What was just done.
    BackupsNotice? notice,

    /// Why the last thing did not happen.
    BackupsProblem? problem,
  }) = _BackupsState;
}
