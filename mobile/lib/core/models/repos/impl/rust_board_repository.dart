// Kanso — rust_board_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/library.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/board_repository.dart';

/// [BoardRepository] over the Rust core's library.
@LazySingleton(as: BoardRepository)
class RustBoardRepository implements BoardRepository {
  RustBoardRepository(this._library);

  final KansoLibrary _library;

  @override
  Future<Either<ApplicationError, BoardView>> board(
    BoardKind kind,
    ListQuery query,
  ) => attempt(() => _library.board(kind: kind, query: query));

  @override
  Future<Either<ApplicationError, CardMoved>> moveCard(
    CardKey card,
    BoardLane lane,
    int nowMs,
  ) => attempt(() => _library.moveCard(card: card, to: lane, nowMs: nowMs));

  @override
  Future<Either<ApplicationError, Unit>> setGroup(String bookId, String? by) =>
      attempt(() async {
        await _library.setBoardGroup(bookId: bookId, by: by);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> setShowUnset(
    String bookId, {
    required bool show,
  }) => attempt(() async {
    await _library.setBoardShowUnset(bookId: bookId, show_: show);
    return unit;
  });

  @override
  Future<Either<ApplicationError, LaneRemoval>> laneRemoval(
    String bookId,
    String lane,
  ) => attempt(() => _library.boardLaneRemoval(bookId: bookId, lane: lane));

  @override
  Future<Either<ApplicationError, LaneEdited>> editLanes(
    String bookId,
    LaneChange change,
    int nowMs,
  ) => attempt(
    () => _library.editBoardLanes(bookId: bookId, change: change, nowMs: nowMs),
  );
}
