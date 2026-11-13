// Kanso — capture_task_cubit.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart' show CaptureTarget;
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/capture_repository.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../models/data/capture_mode.dart';

part 'capture_task_cubit.freezed.dart';
part 'capture_task_state.dart';

/// The capture sheet's Task mode: which mode the sheet is in (kept for the
/// next time), where a captured task goes (also a setting), the line under
/// the field that says how a typed date and repeat read, and saving the task
/// through the library's `capture_task`, the engine code the desktop's Quick
/// Capture runs.
@injectable
class CaptureTaskCubit extends Cubit<CaptureTaskState> {
  CaptureTaskCubit(this._capture, this._preferences)
    : super(const CaptureTaskState());

  final CaptureRepository _capture;
  final PreferencesRepository _preferences;

  /// Reads the saved mode and destination. With [mode] the sheet opens in
  /// it whatever was saved, as the Add task shortcut and tile ask; the
  /// saved choice is left as it is.
  Future<void> started({CaptureMode? mode}) async {
    if (mode != null) {
      // At once, so the sheet does not show the other mode first.
      emit(state.copyWith(mode: mode));
    }
    final loaded = await _preferences.load();
    if (isClosed) {
      return;
    }
    loaded.fold(
      (_) => emit(state.copyWith(loaded: true, mode: mode ?? state.mode)),
      (found) => emit(
        state.copyWith(
          loaded: true,
          mode:
              mode ??
              (found.captureAsTask ? CaptureMode.task : CaptureMode.note),
          target: found.captureTaskTarget,
        ),
      ),
    );
  }

  /// Shows the sheet in [mode] and keeps it for the next time.
  Future<void> modeChosen(CaptureMode mode) async {
    emit(
      state.copyWith(mode: mode, hint: null, status: CaptureTaskStatus.idle),
    );
    await _preferences.update(
      (current) => current.copyWith(captureAsTask: mode == CaptureMode.task),
    );
  }

  /// Changes where captured tasks go, and keeps it.
  Future<void> targetChosen(CaptureTarget target) async {
    emit(state.copyWith(target: target));
    await _preferences.update(
      (current) => current.copyWith(captureTaskTarget: target),
    );
  }

  /// The text in the field changed: reads the date and repeat at its end.
  void textChanged(String text, {required bool monthFirst, DateTime? now}) {
    final hint = _capture.taskHint(
      text,
      now ?? DateTime.now(),
      monthFirst: monthFirst,
    );
    emit(
      state.copyWith(
        hint: hint,
        status: state.status == CaptureTaskStatus.saving
            ? state.status
            : CaptureTaskStatus.idle,
      ),
    );
  }

  /// Saves [text] as a task. Returns whether it was written.
  Future<bool> submitted(
    String text, {
    required bool monthFirst,
    DateTime? now,
  }) async {
    if (state.status == CaptureTaskStatus.saving) {
      return false;
    }
    emit(state.copyWith(status: CaptureTaskStatus.saving));
    final result = await _capture.captureTask(
      text: text,
      target: state.target,
      now: now ?? DateTime.now(),
      monthFirst: monthFirst,
    );
    if (isClosed) {
      return false;
    }
    return result.fold(
      (error) {
        emit(
          state.copyWith(
            status: switch (error) {
              LockedError() => CaptureTaskStatus.locked,
              InvalidError() => CaptureTaskStatus.nothing,
              _ => CaptureTaskStatus.failed,
            },
          ),
        );
        return false;
      },
      (_) {
        emit(state.copyWith(status: CaptureTaskStatus.saved, hint: null));
        return true;
      },
    );
  }
}
