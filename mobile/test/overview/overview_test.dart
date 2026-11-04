// Kanso — overview_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/overview_types.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/repos/abs/ai_repository.dart';
import 'package:kanso/core/models/repos/abs/digest_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/views/widgets/kanso_checkbox.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/overview/state/overview_cubit.dart';
import 'package:kanso/overview/utils/linked_text.dart';
import 'package:kanso/overview/views/pages/overview_page.dart';

import '../helpers/fixture_ai_repository.dart';
import '../helpers/fixture_digest_repository.dart';
import '../helpers/fixture_library_repository.dart';
import '../helpers/pump_routed.dart';

OverviewAgendaRow _row({
  required String title,
  OverviewRowKind kind = OverviewRowKind.due,
  String gutter = 'Today',
  String context = '',
  String noteId = 'note:work',
  int line = 3,
  bool now = false,
  bool isLoop = false,
}) => OverviewAgendaRow(
  kind: kind,
  key: '$title@$line',
  title: title,
  context: context,
  gutter: gutter,
  now: now,
  free: false,
  noteId: noteId,
  line: line,
  isLoop: isLoop,
  raw: title,
  color: '',
  meetingNoteId: '',
);

OverviewListRow _waiting({int quiet = 12}) => OverviewListRow(
  kind: OverviewListKind.waiting,
  noteId: 'note:work',
  line: 5,
  title: 'Rudo',
  detail: 'for the sign-off',
  place: '',
  whenLabel: '',
  quietDays: quiet,
  warn: quiet >= 10,
  key: 'wait-1',
  person: 'note:rudo',
  raw: '',
  isLoop: true,
  caret: false,
  count: 0,
  agent: false,
);

OverviewListRow _pickUp() => const OverviewListRow(
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

OverviewToday _busy({OverviewAi? ai}) => OverviewToday(
  day: '2026-11-04',
  greeting: 'Good morning',
  overline: 'WEDNESDAY 4 NOVEMBER',
  lede:
      'Two meetings and four tasks today, one of them three days late. '
      'Next: Standup in 30 minutes.',
  state: OverviewState.busy,
  earlier: const <OverviewAgendaRow>[],
  agenda: <OverviewAgendaRow>[
    _row(
      title: 'Late thing',
      kind: OverviewRowKind.overdue,
      gutter: '3d late',
      context: 'Work',
    ),
    _row(
      title: 'Standup',
      kind: OverviewRowKind.event,
      gutter: '09:00',
      context: '09:00–09:30 · Google Meet',
      noteId: '',
    ),
    _row(title: 'Send the deck', context: 'Work'),
  ],
  agendaMore: 0,
  taskTotal: 2,
  lists: <OverviewList>[
    OverviewList(
      kind: OverviewListKind.pickUp,
      label: 'Pick up where you left off',
      rows: <OverviewListRow>[_pickUp()],
      more: 0,
    ),
    OverviewList(
      kind: OverviewListKind.waiting,
      label: 'Waiting on others',
      rows: <OverviewListRow>[_waiting()],
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
  ai: ai,
);

const OverviewToday _quiet = OverviewToday(
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

AiSettings _on() => const AiSettings(
  enabled: true,
  provider: 'anthropic',
  host: 'api.anthropic.com',
  hasKey: true,
);

void main() {
  late FixtureDigestRepository digest;
  late FixtureAiRepository ai;
  late FixtureLibraryRepository library;

  setUp(() async {
    await getIt.reset();
    digest = FixtureDigestRepository();
    ai = FixtureAiRepository(settings: _on());
    library = FixtureLibraryRepository();
    getIt
      ..registerSingleton<DigestRepository>(digest)
      ..registerSingleton<AiRepository>(ai)
      ..registerSingleton<LibraryRepository>(library);
    addTearDown(library.close);
    addTearDown(getIt.reset);
  });

  group('linked text', () {
    test('a note link is the note, with or without the prefix', () {
      expect(linkedNote('inkdrop://note/abc'), 'note:abc');
      expect(linkedNote('inkdrop://note/note:abc'), 'note:abc');
      expect(linkedNote('inkdrop://note:abc'), 'note:abc');
      expect(linkedNote('inkdrop://note/'), isNull);
      expect(linkedNote('https://example.com'), isNull);
    });

    test('pieces keep every word, and links to notes open them', () {
      final pieces = linkPieces(
        'See [Plans](inkdrop://note/p1) and [the web](https://x.dev).',
      );
      expect(pieces.map((p) => p.text).join(), 'See Plans and the web.');
      expect(pieces.where((p) => p.noteId != null).single.noteId, 'note:p1');
    });

    test('random text with links never loses a character', () {
      final random = Random(0x6b616e736f);
      const words = <String>['a', 'bc', 'plans', ' ', '.', 'x y'];
      for (var run = 0; run < 200; run++) {
        final plain = StringBuffer();
        final marked = StringBuffer();
        for (var i = 0; i < random.nextInt(8); i++) {
          final word = words[random.nextInt(words.length)];
          if (random.nextBool() && word.trim().isNotEmpty) {
            marked.write('[$word](inkdrop://note/n$i)');
          } else {
            marked.write(word);
          }
          plain.write(word);
        }
        final pieces = linkPieces(marked.toString());
        expect(pieces.map((p) => p.text).join(), plain.toString());
      }
    });
  });

  group('OverviewCubit', () {
    test('draws the snapshot first and then the page', () async {
      digest
        ..snapshot = _quiet
        ..today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      final seen = <OverviewToday?>[];
      cubit.stream.listen((state) => seen.add(state.today));
      await cubit.load();
      expect(seen.first, _quiet, reason: 'the snapshot is the first frame');
      expect(cubit.state.today?.state, OverviewState.busy);
    });

    test('ticking a task checks it and can be undone', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load();
      await cubit.tick(cubit.state.today!.agenda.last);
      expect(library.tasksSet.last, ('note:work', 3, 'Send the deck', true));
      expect(cubit.state.undo?.title, 'Send the deck');
      await cubit.undoTick();
      expect(library.tasksSet.last, ('note:work', 3, 'Send the deck', false));
      expect(cubit.state.undo, isNull);
    });

    test('a promise on the agenda is closed, not ticked', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load();
      await cubit.tick(_row(title: 'Send it', isLoop: true));
      expect(digest.done, <String>['note:work/Send it@3']);
      expect(library.tasksSet, isEmpty);
    });

    test('done, not yet, not now and not useful are each one write', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load();
      final lists = cubit.state.today!.lists;
      await cubit.doneWaiting(lists[1].rows.single);
      await cubit.snoozeWaiting(lists[1].rows.single);
      await cubit.hidePlace(lists[0].rows.single);
      await cubit.dismissPast(cubit.state.today!.past!);
      expect(digest.done, <String>['note:work/wait-1']);
      expect(digest.snoozed, <String>['wait-1']);
      expect(digest.hidden, <String>['note:excalidraw']);
      expect(digest.dismissed, <String>['note:past']);
    });

    test('revisit sets the day and says it', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load();
      await cubit.revisit(
        'note:past',
        const OverviewRevisit(RevisitChoice.nextWeek, 'Mon 9 Nov'),
      );
      expect(digest.revisits, <(String, RevisitChoice)>[
        ('note:past', RevisitChoice.nextWeek),
      ]);
      expect(cubit.state.notice, 'Mon 9 Nov');
    });

    test('summarising asks for the week only on the week', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load();
      await Future<void>.delayed(const Duration(milliseconds: 20));
      await cubit.summarise();
      await cubit.showPerspective(OverviewPerspective.week);
      await cubit.summarise();
      expect(digest.summarised, <(String?, bool)>[
        (null, false),
        ('2026-11-02', true),
      ]);
    });

    test('stepping the week moves it by seven days', () async {
      digest.today = _busy();
      final cubit = OverviewCubit(digest, library, ai);
      addTearDown(cubit.close);
      await cubit.load(perspective: OverviewPerspective.week);
      await cubit.stepWeek(-1);
      expect(cubit.state.weekFrom, '2026-10-26');
      await cubit.stepWeek(2);
      expect(cubit.state.weekFrom, '2026-11-09');
    });
  });

  group('OverviewPage', () {
    testWidgets('shows the day, the agenda and only the lists with rows', (
      tester,
    ) async {
      digest.today = _busy();
      await pumpRouted(tester, const OverviewPage());
      await tester.pumpAndSettle();
      expect(find.text('Good morning'), findsOneWidget);
      expect(find.text('WEDNESDAY 4 NOVEMBER'), findsOneWidget);
      expect(find.text('Late thing'), findsOneWidget);
      expect(find.text('3d late'), findsOneWidget);
      expect(find.text('Standup'), findsOneWidget);
      expect(find.text('Pick up where you left off'), findsOneWidget);
      expect(find.text('12 days quiet'), findsOneWidget);
      expect(find.text('Inbox 7'), findsOneWidget);
      expect(find.text('Sprint retro'), findsOneWidget);
    });

    testWidgets('N more shows the rest of a list in place', (tester) async {
      OverviewListRow place(int i) => OverviewListRow(
        kind: OverviewListKind.pickUp,
        noteId: 'note:p$i',
        line: 0,
        title: 'Place $i',
        detail: '',
        place: 'Kanso',
        whenLabel: '1 h ago',
        quietDays: 0,
        warn: false,
        key: '',
        person: '',
        raw: '',
        isLoop: false,
        caret: false,
        count: 0,
        agent: false,
      );
      final base = _quiet;
      digest.today = OverviewToday(
        day: base.day,
        greeting: base.greeting,
        overline: base.overline,
        lede: base.lede,
        state: OverviewState.busy,
        earlier: const <OverviewAgendaRow>[],
        agenda: const <OverviewAgendaRow>[],
        agendaMore: 0,
        taskTotal: 0,
        lists: <OverviewList>[
          OverviewList(
            kind: OverviewListKind.pickUp,
            label: 'Pick up where you left off',
            // Three shown, then the two that `2 more` holds.
            rows: <OverviewListRow>[for (var i = 0; i < 5; i++) place(i)],
            more: 2,
          ),
        ],
        also: const <OverviewLine>[],
        footer: '',
      );
      await pumpRouted(tester, const OverviewPage());
      await tester.pumpAndSettle();
      expect(find.text('Place 2'), findsOneWidget);
      expect(find.text('Place 3'), findsNothing);
      await tester.tap(find.text('2 more'));
      await tester.pumpAndSettle();
      expect(find.text('Place 3'), findsOneWidget);
      expect(find.text('Place 4'), findsOneWidget);
      expect(find.text('2 more'), findsNothing);
    });

    testWidgets('a quiet day is a short page', (tester) async {
      digest.today = _quiet;
      await pumpRouted(tester, const OverviewPage());
      await tester.pumpAndSettle();
      expect(find.text('Nothing on today.'), findsOneWidget);
      expect(find.text('Waiting on others'), findsNothing);
      expect(find.text('Also'), findsNothing);
    });

    testWidgets('ticking a task asks to be undone', (tester) async {
      digest.today = _busy();
      await pumpRouted(tester, const OverviewPage());
      await tester.pumpAndSettle();
      await tester.tap(find.byType(KansoCheckbox).last);
      await tester.pumpAndSettle();
      expect(library.tasksSet.single.$4, isTrue);
      expect(find.text('Done: Send the deck'), findsOneWidget);
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(library.tasksSet.last.$4, isFalse);
      await tester.pump(overviewUndoFor);
    });

    testWidgets('a citation in the model’s paragraph opens its note', (
      tester,
    ) async {
      digest.today = _busy(
        ai: const OverviewAi(
          text:
              'Pick up [Plans](inkdrop://note/plans) '
              'before the call.',
          timeLabel: '08:41',
        ),
      );
      final router = await pumpRouted(tester, const OverviewPage());
      await tester.pumpAndSettle();
      expect(
        find.textContaining('before the call.', findRichText: true),
        findsOneWidget,
      );
      await tester.tapOnText(find.textRange.ofSubstring('Plans'));
      await tester.pumpAndSettle();
      expect(
        router.routerDelegate.currentConfiguration.uri.toString(),
        contains('note'),
      );
    });

    testWidgets('This week shows the week', (tester) async {
      digest
        ..today = _busy()
        ..week = const OverviewWeek(
          from: '2026-11-02',
          rangeLabel: '2 – 8 NOVEMBER',
          weekLabel: 'Week 45',
          isThisWeek: true,
          lede: 'You wrote 2 notes and finished 1 task.',
          projects: <OverviewProject>[
            OverviewProject(
              id: 'book:a',
              name: 'Kanso',
              notesChanged: 2,
              lastTitle: 'Excalidraw',
              lastNoteId: 'note:excalidraw',
              nextTask: 'Next thing',
              nextDue: 'Fri 6 Nov',
            ),
          ],
          finished: <OverviewFinished>[
            OverviewFinished(noteId: 'note:a', text: 'Shipped it'),
          ],
          finishedMore: 0,
          waiting: <OverviewListRow>[],
          stillOpen: <OverviewAgendaRow>[],
          stillOpenTotal: 0,
          ahead: <OverviewAheadDay>[],
          also: <OverviewLine>[],
          savedAs: '',
        );
      await pumpRouted(tester, const OverviewPage(week: true));
      await tester.pumpAndSettle();
      expect(
        find.text('You wrote 2 notes and finished 1 task.'),
        findsOneWidget,
      );
      expect(find.text('Active projects'), findsOneWidget);
      expect(find.text('Shipped it'), findsOneWidget);
      expect(find.text('Save week as note'), findsOneWidget);
    });

    testWidgets('fits the narrowest phone without overflow', (tester) async {
      digest.today = _busy();
      await pumpRouted(
        tester,
        const OverviewPage(),
        size: const Size(320, 640),
      );
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });
  });
}
