// Kanso — focus_mode_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/repos/abs/preferences_repository.dart';

part 'focus_mode_bloc.freezed.dart';
part 'focus_mode_event.dart';
part 'focus_mode_state.dart';

/// Whether the app shows the open note by itself, without the library, the
/// list, tabs or the note's bars.
///
/// The choice is kept in the preferences, as the desktop keeps it in its
/// layout, so it is still there at the next start. A choice that cannot be
/// saved stays for this run.
@injectable
class FocusModeBloc extends Bloc<FocusModeEvent, FocusModeState> {
  FocusModeBloc(this._preferences) : super(const FocusModeState()) {
    on<FocusModeStartedEvent>(_onStarted, transformer: restartable());
    on<FocusModeToggledEvent>(_onToggled, transformer: sequential());
    on<FocusModeExitedEvent>(_onExited, transformer: sequential());
  }

  final PreferencesRepository _preferences;

  Future<void> _onStarted(
    FocusModeStartedEvent event,
    Emitter<FocusModeState> emit,
  ) async {
    final loaded = await _preferences.load();
    // A choice made while the file was being read is newer than the file.
    if (state.ready) {
      return;
    }
    emit(
      state.copyWith(
        active: loaded.fold((_) => false, (saved) => saved.focusMode),
        ready: true,
      ),
    );
  }

  Future<void> _onToggled(
    FocusModeToggledEvent event,
    Emitter<FocusModeState> emit,
  ) async {
    await _set(!state.active, emit);
  }

  Future<void> _onExited(
    FocusModeExitedEvent event,
    Emitter<FocusModeState> emit,
  ) async {
    await _set(false, emit);
  }

  Future<void> _set(bool active, Emitter<FocusModeState> emit) async {
    if (state.active == active && state.ready) {
      return;
    }
    emit(state.copyWith(active: active, ready: true));
    await _preferences.update(
      (current) => current.copyWith(focusMode: active),
    );
  }
}
