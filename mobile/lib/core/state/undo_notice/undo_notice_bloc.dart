// Kanso — undo_notice_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/data/application_error.dart';
import '../../models/data/undo_notice.dart';
import '../../models/repos/abs/haptics_repository.dart';
import '../../models/repos/abs/library_repository.dart';

part 'undo_notice_bloc.freezed.dart';
part 'undo_notice_event.dart';
part 'undo_notice_state.dart';

/// How long a notice stays, unless a later one replaces it. The shell
/// counts it (see `UndoNoticeTimeout`) and adds [UndoNoticeEvent.timedOut].
const Duration undoNoticeDuration = Duration(seconds: 5);

/// The one notice that offers Undo after a change: a swipe or a choice
/// made on several notes, a ticked task. The shell shows it over every
/// screen; any feature shows one by adding [UndoNoticeEvent.shown].
///
/// A later notice takes the place of the one before, so Undo is always for
/// the latest change. The library takes the change back, as far as it
/// still holds (see `Library::undo`).
@lazySingleton
class UndoNoticeBloc extends Bloc<UndoNoticeEvent, UndoNoticeState> {
  UndoNoticeBloc(this._library, this._haptics)
    : super(const UndoNoticeState()) {
    on<UndoNoticeShownEvent>(_onShown, transformer: sequential());
    on<UndoNoticePressedEvent>(_onUndoPressed, transformer: sequential());
    on<UndoNoticeDismissedEvent>(_onDismissed, transformer: sequential());
    on<UndoNoticeTimedOutEvent>(_onTimedOut, transformer: sequential());
    on<UndoNoticeErrorDismissedEvent>(
      (event, emit) => emit(state.copyWith(error: null)),
      transformer: sequential(),
    );
  }

  final LibraryRepository _library;
  final HapticsRepository _haptics;
  int _serial = 0;

  void _onShown(UndoNoticeShownEvent event, Emitter<UndoNoticeState> emit) {
    emit(
      UndoNoticeState(
        notice: UndoNotice(
          ticket: event.ticket,
          message: event.message,
          serial: ++_serial,
        ),
      ),
    );
  }

  Future<void> _onUndoPressed(
    UndoNoticePressedEvent event,
    Emitter<UndoNoticeState> emit,
  ) async {
    final notice = state.notice;
    if (notice == null || state.undoing) {
      return;
    }
    emit(state.copyWith(undoing: true));
    await _haptics.impact();
    final result = await _library.undo(notice.ticket);
    emit(
      result.fold(
        (error) => UndoNoticeState(error: error),
        (_) => const UndoNoticeState(),
      ),
    );
  }

  void _onTimedOut(
    UndoNoticeTimedOutEvent event,
    Emitter<UndoNoticeState> emit,
  ) {
    if (state.notice?.serial == event.serial && !state.undoing) {
      emit(state.copyWith(notice: null));
    }
  }

  void _onDismissed(
    UndoNoticeDismissedEvent event,
    Emitter<UndoNoticeState> emit,
  ) {
    if (!state.undoing) {
      emit(state.copyWith(notice: null));
    }
  }
}
