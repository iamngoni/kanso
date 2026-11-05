// Kanso — files_migration_state.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'files_migration_bloc.dart';

/// Where the sealing of the attachments of older locked notes is.
enum MigrationStage {
  /// Reading what waits.
  checking,

  /// Nothing to show.
  idle,

  /// The prompt: Seal now, or Later.
  ask,

  /// "Later" was chosen before: a quiet reminder, once.
  remind,

  /// The lock password is asked.
  password,

  /// The typed lock password was wrong.
  wrongPassword,

  /// Sealing (or undoing).
  working,

  /// Done; see the outcome.
  done,

  /// It failed; see the error.
  failed,
}

/// Which of the two the person chose.
enum MigrationAction { seal, undo }

@freezed
abstract class FilesMigrationState with _$FilesMigrationState {
  const factory FilesMigrationState({
    @Default(MigrationStage.checking) MigrationStage stage,
    @Default(MigrationAction.seal) MigrationAction action,

    /// What waits, as last read.
    FilesMigration? info,

    /// Whether the sheet is the first-launch prompt, whose "Later" is
    /// remembered.
    @Default(false) bool launch,

    /// How far sealing has come.
    SealProgress? progress,

    /// Whether Cancel was pressed and the pass is finishing its file.
    @Default(false) bool cancelling,

    /// What a finished pass did.
    MigrationOutcome? outcome,

    /// How many files an undo made readable again.
    @Default(0) int undone,
    ApplicationError? error,
  }) = _FilesMigrationState;
}
