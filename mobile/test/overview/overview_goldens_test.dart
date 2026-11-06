// Kanso — overview_goldens_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
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
  meetingWhich: '',
  meetingDay: '',
  eventJson: '',
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

const OverviewToday _quietDay = OverviewToday(
  day: '2026-11-04',
  greeting: 'Good morning',
  overline: 'WEDNESDAY 4 NOVEMBER',
  lede: 'Nothing on today.',
  state: OverviewState.quiet,
  earlier: <OverviewAgendaRow>[],
  agenda: <OverviewAgendaRow>[],
  agendaMore: 0,
  taskTotal: 0,
  lists: <OverviewList>[],
  also: <OverviewLine>[],
  footer: '',
);

const OverviewToday _newDay = OverviewToday(
  day: '2026-11-04',
  greeting: 'Good morning',
  overline: 'WEDNESDAY 4 NOVEMBER',
  lede: 'Your day will take shape as you write.',
  state: OverviewState.new_,
  earlier: <OverviewAgendaRow>[],
  agenda: <OverviewAgendaRow>[],
  agendaMore: 0,
  taskTotal: 0,
  lists: <OverviewList>[],
  also: <OverviewLine>[],
  footer: '',
);

void main() {
  const calendar = MethodChannel('app.kanso.kanso/system_calendar');
  setUp(() async {
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(calendar, (call) async {
          if (call.method == 'requestAccess') return false;
          if (call.method == 'calendars' || call.method == 'events') {
            return <Object?>[];
          }
          return null;
        });
    addTearDown(
      () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(calendar, null),
    );
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
    for (final (label, day) in <(String, OverviewToday)>[
      ('busy', _day),
      ('quiet', _quietDay),
      ('new', _newDay),
    ]) {
      testWidgets('$label Today on a phone, ${brightness.name}', (
        tester,
      ) async {
        (getIt<DigestRepository>() as FixtureDigestRepository).today = day;
        await expectGoldenOnBothPlatforms(
          tester,
          const OverviewPage(),
          name: 'goldens/overview_${label}_phone_${brightness.name}',
          brightness: brightness,
          size: phoneSize,
        );
      });
    }

    testWidgets('This week on a phone, ${brightness.name}', (tester) async {
      (getIt<DigestRepository>() as FixtureDigestRepository).week =
          OverviewWeek(
            from: '2026-11-02',
            rangeLabel: '2 – 8 NOVEMBER',
            weekLabel: 'Week 45',
            isThisWeek: true,
            lede: 'You wrote 4 notes and finished 2 tasks.',
            projects: <OverviewProject>[
              const OverviewProject(
                id: 'book:kanso',
                name: 'Kanso',
                notesChanged: 4,
                lastTitle: 'Kanso roadmap',
                lastNoteId: 'note:roadmap',
                nextTask: 'Send the beta invite',
                nextDue: 'Thu',
              ),
            ],
            finished: <OverviewFinished>[
              const OverviewFinished(
                noteId: 'note:work',
                text: 'Send the invoice',
              ),
            ],
            finishedMore: 0,
            waiting: <OverviewListRow>[_wait],
            stillOpen: <OverviewAgendaRow>[
              _task('Review the release checklist'),
            ],
            stillOpenTotal: 1,
            ahead: <OverviewAheadDay>[
              OverviewAheadDay(
                summary: 'THURSDAY 5 NOVEMBER',
                rows: <OverviewAgendaRow>[
                  _task('Send the beta invite', gutter: 'Thu'),
                ],
              ),
            ],
            also: <OverviewLine>[
              const OverviewLine(
                kind: OverviewLineKind.themes,
                label: 'Themes',
                count: 3,
                cards: 0,
                items: <OverviewItem>[],
                detail: <String>[],
              ),
            ],
            savedAs: '',
          );
      await expectGoldenOnBothPlatforms(
        tester,
        const OverviewPage(week: true),
        name: 'goldens/overview_week_phone_${brightness.name}',
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
