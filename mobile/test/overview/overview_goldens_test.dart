// Kanso — overview_goldens_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/overview_types.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/overview/views/pages/overview_page.dart';

import '../helpers/fixture_ai_repository.dart';
import '../helpers/fixture_digest_repository.dart';
import '../helpers/fixture_library_repository.dart';
import '../helpers/golden.dart';

OverviewAgendaRow _task(
  String title, {
  OverviewRowKind kind = OverviewRowKind.due,
  String gutter = 'Today',
  String context = '',
  bool now = false,
  String color = '',
}) => OverviewAgendaRow(
  kind: kind,
  key: title,
  title: title,
  context: context,
  gutter: gutter,
  now: now,
  free: false,
  noteId: kind == OverviewRowKind.event ? '' : 'note:work',
  line: 1,
  isLoop: false,
  raw: title,
  color: color,
  meetingNoteId: '',
);

const OverviewListRow _pick = OverviewListRow(
  kind: OverviewListKind.pickUp,
  noteId: 'note:excalidraw',
  line: 4,
  title: 'Excalidraw in Kanso',
  detail: 'and the arrow tool should snap to the shape’s edge, so that',
  place: 'Kanso',
  whenLabel: '2 h ago',
  quietDays: 0,
  warn: false,
  key: '',
  person: '',
  raw: '',
  isLoop: false,
  caret: true,
  count: 0,
  agent: false,
);

const OverviewListRow _wait = OverviewListRow(
  kind: OverviewListKind.waiting,
  noteId: 'note:work',
  line: 5,
  title: 'Rudo',
  detail: 'for the sign-off on the quote',
  place: '',
  whenLabel: '',
  quietDays: 12,
  warn: true,
  key: 'wait-1',
  person: 'note:rudo',
  raw: '',
  isLoop: true,
  caret: false,
  count: 0,
  agent: false,
);

final OverviewToday _day = OverviewToday(
  day: '2026-11-04',
  greeting: 'Good morning',
  overline: 'WEDNESDAY 4 NOVEMBER',
  lede:
      'Two meetings and four tasks today, one of them three days late. '
      'Next: Standup in 30 minutes.',
  state: OverviewState.busy,
  earlier: const <OverviewAgendaRow>[],
  agenda: <OverviewAgendaRow>[
    _task(
      'Send the invoice',
      kind: OverviewRowKind.overdue,
      gutter: '3d late',
      context: 'Work',
    ),
    _task(
      'Standup',
      kind: OverviewRowKind.event,
      gutter: 'Now',
      context: '09:00–09:30 · Google Meet',
      now: true,
      color: '#5e7895',
    ),
    _task(
      'Call with Tendai',
      kind: OverviewRowKind.event,
      gutter: '11:30',
      context: '11:30–12:00 · Tendai Moyo',
      color: '#7bab78',
    ),
    _task('Send the deck', context: 'Work'),
  ],
  agendaMore: 0,
  taskTotal: 2,
  lists: const <OverviewList>[
    OverviewList(
      kind: OverviewListKind.pickUp,
      label: 'Pick up where you left off',
      rows: <OverviewListRow>[_pick],
      more: 0,
    ),
    OverviewList(
      kind: OverviewListKind.waiting,
      label: 'Waiting on others',
      rows: <OverviewListRow>[_wait],
      more: 0,
    ),
  ],
  also: const <OverviewLine>[
    OverviewLine(
      kind: OverviewLineKind.captured,
      label: 'Captured',
      count: 7,
      cards: 0,
      items: <OverviewItem>[],
      detail: <String>[],
    ),
  ],
  past: const OverviewPast(
    noteId: 'note:past',
    title: 'Sprint retro',
    excerpt: 'We keep rewriting the same estimate.',
    why: 'Related to your 11:30 with Tendai',
  ),
  footer: 'Yesterday you wrote 4 notes and finished 6 tasks.',
);

void main() {
  setUp(() async {
    await getIt.reset();
    final library = FixtureLibraryRepository();
    getIt
      ..registerSingleton<DigestRepository>(
        FixtureDigestRepository()..today = _day,
      )
      ..registerSingleton<AiRepository>(FixtureAiRepository())
      ..registerSingleton<LibraryRepository>(library);
    addTearDown(library.close);
    addTearDown(getIt.reset);
  });

  for (final brightness in Brightness.values) {
    testWidgets('Today on a phone, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const OverviewPage(),
        name: 'goldens/overview_today_phone_${brightness.name}',
        brightness: brightness,
        size: phoneSize,
      );
    });
  }

  testWidgets('Today on a tablet, light', (tester) async {
    await expectGoldenOnBothPlatforms(
      tester,
      const OverviewPage(),
      name: 'goldens/overview_today_tablet_light',
      brightness: Brightness.light,
      size: tabletPortraitSize,
    );
  });
}
