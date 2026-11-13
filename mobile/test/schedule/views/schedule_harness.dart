// Kanso — schedule_harness.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/schedule/models/data/schedule_status.dart';
import 'package:kanso/schedule/state/schedule/schedule_bloc.dart';
import 'package:kanso/schedule/views/widgets/schedule_item_tile.dart';
import 'package:kanso/schedule/views/widgets/schedule_pane.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/close_bloc_later.dart';
import '../../helpers/fixture_schedule_repository.dart';

/// Today in the Schedule's fixtures: the day of the mock content.
const DayDate scheduleToday = DayDate(year: 2026, month: 10, day: 26);

/// The Schedule over the fixtures: the shared BLoCs, the rows and the
/// started [bloc].
class ScheduleHarness {
  ScheduleHarness._(this.blocs, this.schedule, this.bloc);

  final BrowseBlocs blocs;
  final FixtureScheduleRepository schedule;
  final ScheduleBloc bloc;

  /// Makes them, with the month read; or only made when [read] is false.
  static Future<ScheduleHarness> start({
    FixtureScheduleRepository? schedule,
    BrowseBlocs? blocs,
    bool read = true,
  }) async {
    final shared = blocs ?? await BrowseBlocs.start();
    final rows = schedule ?? FixtureScheduleRepository();
    final bloc = ScheduleBloc(
      rows,
      shared.library,
      shared.haptics,
      shared.undoNotice,
    );
    closeBlocLater(bloc);
    if (read) {
      bloc.add(
        const ScheduleEvent.started(today: scheduleToday, weekStart: 0),
      );
      await bloc.stream.firstWhere(
        (state) => state.status != ScheduleStatus.loading,
      );
    }
    return ScheduleHarness._(shared, rows, bloc);
  }

  /// The pane, with the BLoCs above it.
  Widget get pane => blocs.provide(
    BlocProvider<ScheduleBloc>.value(
      value: bloc,
      child: const SchedulePane(),
    ),
  );
}

/// The tile of the row titled [title] (the first, if it has several).
Finder tileOf(String title) => find
    .ancestor(
      of: find.text(title).first,
      matching: find.byType(ScheduleItemTile),
    )
    .first;

/// The more button inside [tile].
Finder moreOf(Finder tile) => find.descendant(
  of: tile,
  matching: find.byWidgetPredicate(
    (widget) =>
        widget is KansoIconButton && widget.semanticLabel == 'More actions',
  ),
);

/// The button of the bar or a sheet called [label].
Finder buttonLabelled(String label) => find.byWidgetPredicate(
  (widget) => widget is KansoIconButton && widget.semanticLabel == label,
);
