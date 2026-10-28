// Kanso — board_drag.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../../core/bridge/api/types.dart';

/// A card being dragged to another column (on a tablet): which card, and
/// the column it is dragged from.
class BoardDrag {
  const BoardDrag({required this.card, required this.from});

  final CardKey card;
  final BoardLane from;
}
