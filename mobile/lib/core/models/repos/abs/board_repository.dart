// Kanso — board_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// Boards over the notes a list shows (the trash never shows).
abstract interface class BoardRepository {
  /// The columns of a [kind] board over what [query] lists.
  Future<Either<ApplicationError, BoardView>> board(
    BoardKind kind,
    ListQuery query,
  );

  /// Moves [card] to [lane]: sets a note's status, or ticks or unticks a
  /// task. [nowMs] is the time of the change, in milliseconds since the
  /// epoch.
  Future<Either<ApplicationError, CardMoved>> moveCard(
    CardKey card,
    BoardLane lane,
    int nowMs,
  );

  /// Groups notebook [bookId]'s board by the select property [by], or by
  /// status when it is null.
  Future<Either<ApplicationError, Unit>> setGroup(String bookId, String? by);

  /// Shows or hides the lane of notes with no value while it is empty.
  Future<Either<ApplicationError, Unit>> setShowUnset(
    String bookId, {
    required bool show,
  });

  /// What removing [lane] asks first: how many notes are in it, the words
  /// that ask, and where they go unless another lane is chosen.
  Future<Either<ApplicationError, LaneRemoval>> laneRemoval(
    String bookId,
    String lane,
  );

  /// Renames, reorders, adds or removes a lane. A refused edit is an
  /// invalid error whose message is the core's words for why.
  Future<Either<ApplicationError, LaneEdited>> editLanes(
    String bookId,
    LaneChange change,
    int nowMs,
  );
}
