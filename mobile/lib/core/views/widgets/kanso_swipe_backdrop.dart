// Kanso — kanso_swipe_backdrop.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../models/data/kanso_swipe_action.dart';
import '../../utils/swipe_outcome.dart';
import 'kanso_swipe_action_tile.dart';

/// The actions behind a row that has been swiped [shown] points: they
/// appear from the edge as the row moves away, each as wide as an action is
/// until the swipe is wider than all of them, and then the outer one takes
/// the rest.
///
/// [actions] are in the order they are laid out in, from the start; with
/// [atStart] the outer action is the first, otherwise the last.
class KansoSwipeBackdrop extends StatelessWidget {
  const KansoSwipeBackdrop({
    required this.actions,
    required this.shown,
    required this.atStart,
    super.key,
  });

  final List<KansoSwipeAction> actions;
  final double shown;
  final bool atStart;

  @override
  Widget build(BuildContext context) {
    final full = math.max(shown, SwipeThresholds.actionWidth * actions.length);
    final outer = atStart ? 0 : actions.length - 1;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisAlignment: atStart
          ? MainAxisAlignment.start
          : MainAxisAlignment.end,
      children: <Widget>[
        SizedBox(
          width: shown,
          child: ClipRect(
            child: OverflowBox(
              minWidth: full,
              maxWidth: full,
              alignment: atStart
                  ? AlignmentDirectional.centerStart
                  : AlignmentDirectional.centerEnd,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  for (var i = 0; i < actions.length; i++)
                    if (i == outer)
                      Expanded(child: KansoSwipeActionTile(action: actions[i]))
                    else
                      SizedBox(
                        width: SwipeThresholds.actionWidth,
                        child: KansoSwipeActionTile(action: actions[i]),
                      ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}
