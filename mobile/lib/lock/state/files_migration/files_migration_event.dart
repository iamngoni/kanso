// Kanso — files_migration_event.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'files_migration_bloc.dart';

@freezed
sealed class FilesMigrationEvent with _$FilesMigrationEvent {
  /// Look at what waits to be sealed. At [launch], decide whether to ask or
  /// remind; elsewhere only read it.
  const factory FilesMigrationEvent.checked({@Default(false) bool launch}) =
      FilesMigrationCheckedEvent;

  /// The sheet is opened on purpose (the Settings row).
  const factory FilesMigrationEvent.opened() = FilesMigrationOpenedEvent;

  /// "Seal now" or "Finish sealing" was chosen.
  const factory FilesMigrationEvent.sealRequested() =
      FilesMigrationSealRequestedEvent;

  /// Seal, with [password] when the session holds no key.
  const factory FilesMigrationEvent.sealed({String? password}) =
      FilesMigrationSealedEvent;

  /// "Undo" was chosen.
  const factory FilesMigrationEvent.undoRequested() =
      FilesMigrationUndoRequestedEvent;

  /// Undo, with [password] when the session holds no key.
  const factory FilesMigrationEvent.undone({String? password}) =
      FilesMigrationUndoneEvent;

  /// "Later" was chosen, or the sheet was dismissed.
  const factory FilesMigrationEvent.postponed() = FilesMigrationPostponedEvent;

  /// Cancel was pressed while sealing.
  const factory FilesMigrationEvent.cancelled() = FilesMigrationCancelledEvent;

  /// Sealing got as far as [progress].
  const factory FilesMigrationEvent.progressed(SealProgress progress) =
      FilesMigrationProgressedEvent;

  /// The result was shown.
  const factory FilesMigrationEvent.dismissed() = FilesMigrationDismissedEvent;
}
