// Kanso — board_labels.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../../core/utils/status_label.dart';
import '../../gen/l10n/app_localizations.dart';

/// The words for the columns of a board.
extension BoardLaneLabel on AppLocalizations {
  /// The name of [column], in the app's language; the core's own name for a
  /// status this version has no word for.
  String laneName(BoardColumn column) {
    return switch (column.lane) {
      BoardLane_Todo() => boardLaneTodo,
      BoardLane_Done() => boardLaneDone,
      BoardLane_Status(:final status) =>
        status == 'none' || status.isEmpty
            ? boardLaneNoStatus
            : _statusName(status, column.label),
      // A property board words its own lanes (“No stage”): the core knows
      // the property's name, as it does on the desktop.
      BoardLane_Value(:final value) => value,
      BoardLane_Unset() || BoardLane_Unlisted() => column.label,
    };
  }

  String _statusName(String status, String fallback) {
    final name = statusName(noteStatusOf(status));
    return name.isEmpty ? fallback : name;
  }
}
