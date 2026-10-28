// Kanso — reminders_settings_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/data/reminder_permission.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../core/models/repos/abs/reminder_notifier.dart';
import '../../utils/reminders_coordinator.dart';

part 'reminders_settings_bloc.freezed.dart';
part 'reminders_settings_event.dart';
part 'reminders_settings_state.dart';

/// The Settings row "Remind me of tasks": the switch (kept in the
/// preferences, on unless turned off) and whether the system lets Kanso
/// show notifications.
///
/// Turning the switch on asks the system for permission if it has not been
/// asked. A permission that was refused stays refused until the person
/// allows it in the system's settings; the row says so.
@injectable
class RemindersSettingsBloc
    extends Bloc<RemindersSettingsEvent, RemindersSettingsState> {
  RemindersSettingsBloc(
    this._preferences,
    this._notifier,
    this._coordinator,
  ) : super(const RemindersSettingsState()) {
    on<RemindersSettingsStartedEvent>(_onStarted, transformer: restartable());
    on<RemindersSettingsResumedEvent>(_onResumed, transformer: restartable());
    on<RemindersSettingsToggledEvent>(_onToggled, transformer: sequential());
  }

  final PreferencesRepository _preferences;
  final ReminderNotifier _notifier;
  final RemindersCoordinator _coordinator;

  Future<void> _onStarted(
    RemindersSettingsStartedEvent event,
    Emitter<RemindersSettingsState> emit,
  ) async {
    final saved = await _preferences.load();
    final permission = await _notifier.permission();
    if (emit.isDone) {
      return;
    }
    emit(
      saved.fold(
        (error) => RemindersSettingsState(
          status: LoadStatus.failed,
          permission: permission,
          error: error,
        ),
        (preferences) => RemindersSettingsState(
          status: LoadStatus.ready,
          enabled: preferences.remindersEnabled,
          permission: permission,
        ),
      ),
    );
  }

  /// The app came back, perhaps from the system's settings.
  Future<void> _onResumed(
    RemindersSettingsResumedEvent event,
    Emitter<RemindersSettingsState> emit,
  ) async {
    final permission = await _notifier.permission();
    if (!emit.isDone) {
      emit(state.copyWith(permission: permission));
    }
  }

  Future<void> _onToggled(
    RemindersSettingsToggledEvent event,
    Emitter<RemindersSettingsState> emit,
  ) async {
    // The person may have changed it in the system's settings meanwhile.
    var permission = await _notifier.permission();
    if (event.enabled && permission == ReminderPermission.notDetermined) {
      final allowed = await _notifier.requestPermission();
      permission = allowed
          ? ReminderPermission.granted
          : await _notifier.permission();
      // An answer that is neither yes nor "not asked" is a no.
      if (!allowed && permission == ReminderPermission.notDetermined) {
        permission = ReminderPermission.denied;
      }
    }
    final saved = await _preferences.update(
      (current) => current.copyWith(remindersEnabled: event.enabled),
    );
    if (emit.isDone) {
      return;
    }
    saved.fold(
      (error) => emit(state.copyWith(permission: permission, error: error)),
      (preferences) => emit(
        state.copyWith(
          enabled: preferences.remindersEnabled,
          permission: permission,
          error: null,
        ),
      ),
    );
    await _coordinator.refresh();
  }
}
