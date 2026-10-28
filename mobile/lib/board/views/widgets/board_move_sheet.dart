// Kanso — board_move_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../utils/board_labels.dart';

/// The columns of the board, to move a card to; the one it is in is
/// checked. Choosing another closes the sheet with its lane.
class BoardMoveSheet extends StatelessWidget {
  const BoardMoveSheet({
    required this.columns,
    required this.current,
    super.key,
  });

  final List<BoardColumn> columns;
  final BoardLane current;

  /// Shows the sheet over the whole app; completes with the lane chosen, or
  /// null.
  static Future<BoardLane?> show(
    BuildContext context, {
    required List<BoardColumn> columns,
    required BoardLane current,
  }) {
    return showKansoBottomSheet<BoardLane>(
      context,
      builder: (_) => BoardMoveSheet(columns: columns, current: current),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      children: <Widget>[
        SheetHeader(
          title: l10n.boardMoveSheetTitle,
          actionLabel: l10n.searchCancel,
        ),
        for (final column in columns)
          KansoCheckRow(
            label: l10n.laneName(column),
            checked: column.lane == current,
            onTap: () => Navigator.of(context).pop(column.lane),
          ),
      ],
    );
  }
}
