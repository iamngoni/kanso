// Kanso — sync_settings_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/sync_repository.dart';

part 'sync_settings_bloc.freezed.dart';
part 'sync_settings_event.dart';
part 'sync_settings_state.dart';

/// When sync runs on its own: Sync automatically, and how often, from the
/// desktop's choices. The library keeps them; the schedule and the
/// background sync follow at once.
///
/// A change is shown at once and then saved; one that cannot be saved goes
/// back to what the library has, and says why.
@injectable
class SyncSettingsBloc extends Bloc<SyncSettingsEvent, SyncSettingsState> {
  SyncSettingsBloc(this._sync) : super(const SyncSettingsState()) {
    on<SyncSettingsEvent>(
      (event, emit) => switch (event) {
        SyncSettingsStartedEvent() => _onStarted(emit),
        SyncSettingsEnabledChangedEvent(:final enabled) => _change(
          emit,
          enabled: enabled,
        ),
        SyncSettingsIntervalChosenEvent(:final minutes) => _change(
          emit,
          intervalSecs: minutes * 60,
        ),
      },
      transformer: sequential(),
    );
  }

  final SyncRepository _sync;

  /// The desktop's status line for [status] under the settings as shown,
  /// or null until they are read. [when] and [resume] are the shell's
  /// words for the last sync and for when a pause ends.
  SyncStatusLine? statusLine(
    SyncStatus status, {
    required String when,
    required String resume,
  }) {
    final settings = state.settings;
    if (settings == null) {
      return null;
    }
    return _sync.statusLine(status, settings, when: when, resume: resume);
  }

  Future<void> _onStarted(Emitter<SyncSettingsState> emit) async {
    final options = _sync.intervalOptions();
    final read = await _sync.settings();
    emit(
      read.fold(
        (error) => SyncSettingsState(
          status: LoadStatus.failed,
          options: options,
          error: error,
        ),
        (settings) => SyncSettingsState(
          status: LoadStatus.ready,
          settings: settings,
          options: options,
        ),
      ),
    );
  }

  Future<void> _change(
    Emitter<SyncSettingsState> emit, {
    bool? enabled,
    int? intervalSecs,
  }) async {
    final before = state.settings;
    if (before == null) {
      return;
    }
    final wanted = SyncSettingsRow(
      enabled: enabled ?? before.enabled,
      intervalSecs: intervalSecs ?? before.intervalSecs,
    );
    if (wanted == before) {
      return;
    }
    emit(state.copyWith(settings: wanted, error: null));
    final saved = await _sync.setSettings(
      enabled: wanted.enabled,
      intervalSecs: wanted.intervalSecs,
    );
    emit(
      saved.fold(
        (error) => state.copyWith(settings: before, error: error),
        (settings) => state.copyWith(settings: settings),
      ),
    );
  }
}
