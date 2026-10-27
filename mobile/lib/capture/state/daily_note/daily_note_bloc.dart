// Kanso — daily_note_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/capture_repository.dart';

part 'daily_note_bloc.freezed.dart';
part 'daily_note_event.dart';
part 'daily_note_state.dart';

/// Finds today's daily note for `kanso://open/daily`, making it if it isn't
/// there, so the screen can go to it.
@injectable
class DailyNoteBloc extends Bloc<DailyNoteEvent, DailyNoteState> {
  DailyNoteBloc(this._capture) : super(const DailyNoteState()) {
    on<DailyNoteRequestedEvent>(_onRequested, transformer: restartable());
  }

  final CaptureRepository _capture;

  Future<void> _onRequested(
    DailyNoteRequestedEvent event,
    Emitter<DailyNoteState> emit,
  ) async {
    emit(const DailyNoteState());
    final result = await _capture.todayNote(event.now);
    emit(
      result.fold(
        (error) => DailyNoteState(status: LoadStatus.failed, error: error),
        (id) => DailyNoteState(status: LoadStatus.ready, noteId: id),
      ),
    );
  }
}
