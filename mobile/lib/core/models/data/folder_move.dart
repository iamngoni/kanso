// Kanso — folder_move.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the move of the library folder into the App Group container does,
/// from whether each folder has a library in it.
enum FolderMoveDecision {
  /// The old folder's library goes to the new place.
  move,

  /// Both have a library: the new one is used and the old one is left alone.
  keepCurrent,

  /// Nothing to move: the new place has its library, or neither has one.
  nothing,
}

/// What a move did.
enum FolderMoveOutcome {
  /// The old folder's contents were renamed into the new place.
  moved,

  /// Some contents could not be renamed and were copied, then removed from
  /// the old place.
  copied,

  /// Both places had a library; the new one is used and the old is as it was.
  keptCurrent,

  /// There was nothing to move.
  nothingToMove,
}

/// Whether to move, from whether the new place ([currentHasLibrary]) and
/// the old one ([oldHasLibrary]) hold a `library.db`.
///
/// Only a library with nowhere to go yet moves; a library in the new place
/// is never replaced.
FolderMoveDecision decideFolderMove({
  required bool currentHasLibrary,
  required bool oldHasLibrary,
}) {
  if (!oldHasLibrary) {
    return FolderMoveDecision.nothing;
  }
  return currentHasLibrary
      ? FolderMoveDecision.keepCurrent
      : FolderMoveDecision.move;
}
