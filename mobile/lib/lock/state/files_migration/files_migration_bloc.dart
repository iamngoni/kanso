// Kanso — files_migration_bloc.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../utils/files_migration_words.dart';

part 'files_migration_bloc.freezed.dart';
part 'files_migration_event.dart';
part 'files_migration_state.dart';

/// Sealing the attachments of notes locked before attachments were sealed
/// (#44, step 7): the prompt at the first launch after the update, the
/// Settings row that does it later, and the pass itself with its progress.
///
/// Nothing is sealed without the lock password: when the session holds no key
/// the sheet asks for it. Cancel stops the pass between two files and
/// running it again finishes; a pass that stopped can also be undone.
@injectable
class FilesMigrationBloc
    extends Bloc<FilesMigrationEvent, FilesMigrationState> {
  FilesMigrationBloc(this._session) : super(const FilesMigrationState()) {
    on<FilesMigrationCheckedEvent>(_onChecked, transformer: restartable());
    on<FilesMigrationOpenedEvent>(_onOpened);
    on<FilesMigrationSealRequestedEvent>(_onSealRequested);
    on<FilesMigrationSealedEvent>(_onSealed, transformer: droppable());
    on<FilesMigrationUndoRequestedEvent>(_onUndoRequested);
    on<FilesMigrationUndoneEvent>(_onUndone, transformer: droppable());
    on<FilesMigrationPostponedEvent>(_onPostponed);
    on<FilesMigrationCancelledEvent>(_onCancelled);
    on<FilesMigrationProgressedEvent>(
      (event, emit) => emit(state.copyWith(progress: event.progress)),
    );
    on<FilesMigrationDismissedEvent>(
      (event, emit) => emit(
        state.copyWith(stage: MigrationStage.idle, outcome: null, error: null),
      ),
    );
  }

  final SessionRepository _session;

  Future<void> _onChecked(
    FilesMigrationCheckedEvent event,
    Emitter<FilesMigrationState> emit,
  ) async {
    final found = await _session.filesMigration();
    found.fold(
      (error) => emit(state.copyWith(stage: MigrationStage.idle)),
      (info) => emit(
        state.copyWith(
          info: info,
          launch: event.launch,
          stage: event.launch
              ? wantsMigrationPrompt(info)
                    ? MigrationStage.ask
                    : wantsMigrationReminder(info)
                    ? MigrationStage.remind
                    : MigrationStage.idle
              : state.stage == MigrationStage.checking
              ? MigrationStage.idle
              : state.stage,
        ),
      ),
    );
  }

  /// The Settings row was tapped: the sheet opens on what waits.
  void _onOpened(
    FilesMigrationOpenedEvent event,
    Emitter<FilesMigrationState> emit,
  ) {
    if ((state.info?.notes ?? 0) > 0) {
      emit(
        state.copyWith(
          stage: MigrationStage.ask,
          launch: false,
          outcome: null,
          error: null,
          progress: null,
        ),
      );
    }
  }

  /// "Seal now" (or "Finish sealing"): straight to the work with a key held,
  /// else the password first.
  void _onSealRequested(
    FilesMigrationSealRequestedEvent event,
    Emitter<FilesMigrationState> emit,
  ) {
    if (state.info?.counted ?? false) {
      add(const FilesMigrationEvent.sealed());
    } else {
      emit(
        state.copyWith(
          stage: MigrationStage.password,
          action: MigrationAction.seal,
          error: null,
        ),
      );
    }
  }

  void _onUndoRequested(
    FilesMigrationUndoRequestedEvent event,
    Emitter<FilesMigrationState> emit,
  ) {
    emit(
      state.copyWith(
        stage: MigrationStage.password,
        action: MigrationAction.undo,
        error: null,
      ),
    );
  }

  Future<void> _onSealed(
    FilesMigrationSealedEvent event,
    Emitter<FilesMigrationState> emit,
  ) async {
    emit(
      state.copyWith(
        stage: MigrationStage.working,
        action: MigrationAction.seal,
        error: null,
        progress: null,
        cancelling: false,
      ),
    );
    final listening = _session.lockProgress().listen((progress) {
      if (!isClosed) {
        add(FilesMigrationEvent.progressed(progress));
      }
    });
    final Either<ApplicationError, MigrationOutcome> result;
    try {
      result = await _session.sealLockedAttachments(password: event.password);
    } finally {
      unawaited(listening.cancel());
    }
    emit(
      result.fold(
        (error) => _failed(error, event.password),
        (outcome) => state.copyWith(
          stage: MigrationStage.done,
          outcome: outcome,
          progress: null,
          cancelling: false,
        ),
      ),
    );
    if (result.isRight()) {
      add(const FilesMigrationEvent.checked());
    }
  }

  Future<void> _onUndone(
    FilesMigrationUndoneEvent event,
    Emitter<FilesMigrationState> emit,
  ) async {
    emit(
      state.copyWith(
        stage: MigrationStage.working,
        action: MigrationAction.undo,
        error: null,
        progress: null,
      ),
    );
    final result = await _session.undoFilesMigration(password: event.password);
    emit(
      result.fold(
        (error) => _failed(error, event.password),
        (files) => state.copyWith(
          stage: MigrationStage.done,
          undone: files,
          outcome: null,
        ),
      ),
    );
    if (result.isRight()) {
      add(const FilesMigrationEvent.checked());
    }
  }

  /// Said no key is held, a wrong password, or a failure.
  FilesMigrationState _failed(ApplicationError error, String? password) {
    return switch (error) {
      LockedError() => state.copyWith(stage: MigrationStage.password),
      AuthError() || InvalidError() when password != null => state.copyWith(
        stage: MigrationStage.wrongPassword,
      ),
      _ => state.copyWith(stage: MigrationStage.failed, error: error),
    };
  }

  /// "Later". At the first launch it is remembered, so the next launches
  /// remind quietly instead of asking again.
  Future<void> _onPostponed(
    FilesMigrationPostponedEvent event,
    Emitter<FilesMigrationState> emit,
  ) async {
    emit(state.copyWith(stage: MigrationStage.idle));
    if (state.launch) {
      await _session.postponeFilesMigration();
    }
  }

  Future<void> _onCancelled(
    FilesMigrationCancelledEvent event,
    Emitter<FilesMigrationState> emit,
  ) async {
    emit(state.copyWith(cancelling: true));
    await _session.cancelLock();
  }
}
