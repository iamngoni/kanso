// Kanso — board_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:clock/clock.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/board_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/utils/debounce.dart';

part 'board_bloc.freezed.dart';
part 'board_event.dart';
part 'board_state.dart';

/// How long the writes of a burst may be apart for the board to be read
/// again once, after the last.
const Duration boardDebounce = Duration(milliseconds: 100);

/// A board over the notes a list shows: their columns, read from the core,
/// kept up to date as the library changes, and changed by moving a card.
///
/// After a move the board is read again (the core is the one that knows
/// where the card is now); a move that fails says so and the board is read
/// again too, so it never shows a card where it is not.
@injectable
class BoardBloc extends Bloc<BoardEvent, BoardState> {
  BoardBloc(this._board, this._library) : super(const BoardState()) {
    on<BoardStartedEvent>(_onStarted, transformer: restartable());
    on<BoardKindChangedEvent>(_onKindChanged, transformer: restartable());
    on<BoardChangesReceivedEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<BoardChangesReceivedEvent>(
        boardDebounce,
      ),
    );
    on<BoardRetriedEvent>((event, emit) => _read(emit));
    on<BoardCardMovedEvent>(_onMoved, transformer: sequential());
    on<BoardNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(notice: null)),
    );
  }

  final BoardRepository _board;
  final LibraryRepository _library;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  Future<void> _onStarted(
    BoardStartedEvent event,
    Emitter<BoardState> emit,
  ) async {
    emit(BoardState(query: event.query, kind: event.kind));
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(BoardEvent.changesReceived(ids)),
    );
    await _read(emit);
  }

  Future<void> _onKindChanged(
    BoardKindChangedEvent event,
    Emitter<BoardState> emit,
  ) async {
    if (event.kind == state.kind) {
      return;
    }
    emit(
      BoardState(
        query: state.query,
        kind: event.kind,
      ),
    );
    await _read(emit);
  }

  /// Reads the board. A reading that a later one overtook is dropped, so
  /// the state never goes back to older data.
  Future<void> _read(Emitter<BoardState> emit) async {
    final query = state.query;
    final kind = state.kind;
    if (query == null) {
      return;
    }
    final reading = ++_reading;
    final result = await _board.board(kind, query);
    if (reading != _reading || emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(
        state.copyWith(
          status: LoadStatus.failed,
          error: error,
        ),
      ),
      (data) => emit(
        state.copyWith(status: LoadStatus.ready, data: data, error: null),
      ),
    );
  }

  Future<void> _onMoved(
    BoardCardMovedEvent event,
    Emitter<BoardState> emit,
  ) async {
    final result = await _board.moveCard(
      event.card,
      event.lane,
      clock.now().millisecondsSinceEpoch,
    );
    if (emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(notice: BoardNotice.failed(error))),
      (moved) => switch (moved) {
        CardMoved.changed || CardMoved.unchanged => emit(
          state.copyWith(notice: null),
        ),
        CardMoved.missing => emit(
          state.copyWith(notice: const BoardNotice.changedInNote()),
        ),
      },
    );
    await _read(emit);
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
