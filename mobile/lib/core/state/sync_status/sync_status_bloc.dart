// Kanso — sync_status_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../bridge/api/types.dart';
import '../../models/data/application_error.dart';
import '../../models/repos/abs/sync_repository.dart';

part 'sync_status_bloc.freezed.dart';
part 'sync_status_event.dart';
part 'sync_status_state.dart';

/// The state of the account and its sync, for the settings row and the pull
/// to refresh, which exists only while an account is signed in.
@injectable
class SyncStatusBloc extends Bloc<SyncStatusEvent, SyncStatusState> {
  SyncStatusBloc(this._sync) : super(const SyncStatusState()) {
    on<SyncStatusStartedEvent>(_onStarted, transformer: restartable());
    on<SyncStatusSyncRequestedEvent>(
      _onSyncRequested,
      transformer: droppable(),
    );
    on<SyncStatusForegroundChangedEvent>(
      _onForegroundChanged,
      transformer: sequential(),
    );
  }

  final SyncRepository _sync;

  Future<void> _onStarted(
    SyncStatusStartedEvent event,
    Emitter<SyncStatusState> emit,
  ) {
    return emit.onEach<SyncStatus>(
      _sync.statusStream(),
      onData: (status) => emit(state.copyWith(status: status)),
    );
  }

  Future<void> _onForegroundChanged(
    SyncStatusForegroundChangedEvent event,
    Emitter<SyncStatusState> emit,
  ) async {
    // A failure to say so changes nothing the person can act on: the library
    // keeps its own state, and the next change says it again.
    await _sync.setForeground(foreground: event.foreground);
  }

  Future<void> _onSyncRequested(
    SyncStatusSyncRequestedEvent event,
    Emitter<SyncStatusState> emit,
  ) async {
    emit(state.copyWith(syncing: true));
    final result = await _sync.syncNow();
    emit(
      result.fold(
        (error) => state.copyWith(syncing: false, error: error),
        (_) => state.copyWith(syncing: false, error: null),
      ),
    );
  }
}
