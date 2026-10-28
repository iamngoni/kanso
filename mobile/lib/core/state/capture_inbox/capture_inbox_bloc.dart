// Kanso — capture_inbox_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/application_error.dart';
import '../../models/repos/abs/capture_repository.dart';

part 'capture_inbox_bloc.freezed.dart';
part 'capture_inbox_event.dart';
part 'capture_inbox_state.dart';

/// Saves what other apps shared into Kanso (the share extension, the
/// shortcuts, the share activity), as notes in the Inbox.
///
/// It is asked when the app starts and each time it comes back to the
/// front. Two saves never overlap: one asked for while another runs is
/// dropped, and the next time the app comes back saves what it missed. The
/// lists read again after every save (see [CaptureRepository.saveDrops]).
@injectable
class CaptureInboxBloc extends Bloc<CaptureInboxEvent, CaptureInboxState> {
  CaptureInboxBloc(this._capture) : super(const CaptureInboxState()) {
    on<CaptureInboxSaveRequestedEvent>(
      _onSaveRequested,
      transformer: droppable(),
    );
    on<CaptureInboxProblemsDismissedEvent>(
      (event, emit) => emit(state.copyWith(problems: const <String>[])),
    );
  }

  final CaptureRepository _capture;

  Future<void> _onSaveRequested(
    CaptureInboxSaveRequestedEvent event,
    Emitter<CaptureInboxState> emit,
  ) async {
    emit(state.copyWith(saving: true));
    final result = await _capture.saveDrops();
    emit(
      result.fold(
        (error) => state.copyWith(saving: false, error: error),
        (saved) => state.copyWith(
          saving: false,
          error: null,
          saved: saved.notes,
          savedCount: state.savedCount + saved.notes.length,
          kept: saved.kept,
          problems: <String>[...state.problems, ...saved.problems],
        ),
      ),
    );
  }
}
