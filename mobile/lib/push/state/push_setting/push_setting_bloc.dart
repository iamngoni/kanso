// Kanso — push_setting_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../models/data/push_setting_status.dart';
import '../../models/repos/abs/push_repository.dart';

part 'push_setting_bloc.freezed.dart';
part 'push_setting_event.dart';
part 'push_setting_state.dart';

/// The setting "Wake this device when notes change elsewhere": on until the
/// person turns it off, which removes the token from the server.
///
/// A change shows at once and is put back if it can't be made.
@injectable
class PushSettingBloc extends Bloc<PushSettingEvent, PushSettingState> {
  PushSettingBloc(this._push) : super(const PushSettingState()) {
    on<PushSettingStartedEvent>(_onStarted, transformer: restartable());
    on<PushSettingToggledEvent>(_onToggled, transformer: droppable());
    on<PushSettingErrorDismissedEvent>(
      (event, emit) => emit(state.copyWith(error: null)),
    );
  }

  final PushRepository _push;

  Future<void> _onStarted(
    PushSettingStartedEvent event,
    Emitter<PushSettingState> emit,
  ) async {
    final reason = await _push.unavailableReason();
    if (reason.fold((_) => true, (why) => why != null)) {
      emit(const PushSettingState(status: PushSettingStatus.unavailable));
      return;
    }
    final enabled = await _push.isEnabled();
    emit(
      enabled.fold(
        // A setting that can't be read is the default: on.
        (error) => PushSettingState(
          status: PushSettingStatus.ready,
          error: error,
        ),
        (value) => PushSettingState(
          status: PushSettingStatus.ready,
          enabled: value,
        ),
      ),
    );
  }

  Future<void> _onToggled(
    PushSettingToggledEvent event,
    Emitter<PushSettingState> emit,
  ) async {
    if (state.status != PushSettingStatus.ready) {
      return;
    }
    final before = state.enabled;
    emit(state.copyWith(enabled: event.enabled, saving: true, error: null));
    final result = await _push.setEnabled(enabled: event.enabled);
    emit(
      result.fold(
        (error) => state.copyWith(enabled: before, saving: false, error: error),
        (_) => state.copyWith(saving: false),
      ),
    );
  }
}
