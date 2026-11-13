// Kanso — schedule_goldens_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/schedule/models/data/schedule_status.dart';
import 'package:kanso/schedule/state/schedule/schedule_bloc.dart';

import '../../helpers/golden.dart';
import 'schedule_harness.dart';

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('the month view, ${brightness.name}', (tester) async {
      final harness = await ScheduleHarness.start();
      await expectGoldenOnBothPlatforms(
        tester,
        harness.pane,
        name: 'phone_schedule_month_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('the agenda view, ${brightness.name}', (tester) async {
      final harness = await ScheduleHarness.start();
      harness.bloc.add(const ScheduleEvent.modeChanged(ScheduleMode.agenda));
      // Real time, so the mode change is handled before the golden pumps.
      await tester.runAsync(pumpEventQueue);
      await expectGoldenOnBothPlatforms(
        tester,
        harness.pane,
        name: 'phone_schedule_agenda_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
