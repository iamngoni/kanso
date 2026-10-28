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
}
