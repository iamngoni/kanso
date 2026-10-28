// Kanso — palette_launcher_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/journal_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/graph/state/graph/graph_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/views/widgets/calendar_sheet.dart';
import 'package:kanso/note_links/models/repos/abs/note_links_repository.dart';
import 'package:kanso/note_links/views/widgets/outline_sheet.dart';
import 'package:kanso/palette/models/repos/abs/palette_repository.dart';
import 'package:kanso/palette/state/palette/palette_bloc.dart';
import 'package:kanso/sharing/views/widgets/share_note_sheet.dart';

import '../../helpers/fixture_discovery_repository.dart';
import '../../helpers/fixture_journal_repository.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';

const Size _size = Size(1180, 820);
const String _noteId = 'note:palette-test';
const String _noteRoute = '/notes/note/note%3Apalette-test';

NoteView _routeNote() => NoteView(
  id: _noteId,
  rev: mockOpenNote.rev,
  title: mockOpenNote.title,
  body: mockOpenNote.body,
  lock: mockOpenNote.lock,
  notebookId: mockOpenNote.notebookId,
  tags: mockOpenNote.tags,
  status: mockOpenNote.status,
  pinned: mockOpenNote.pinned,
  createdMs: mockOpenNote.createdMs,
  updatedMs: mockOpenNote.updatedMs,
);

class _Library extends FixtureLibraryRepository {
  final List<String> noteReads = <String>[];

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) {
    noteReads.add(id);
    return super.note(id);
  }
}

class _Links implements NoteLinksRepository {
  _Links() {
    latest = this;
  }

  static late _Links latest;

  final List<String> outlines = <String>[];

  @override
  Either<ApplicationError, List<OutlineHeading>> outline(String text) {
    outlines.add(text);
    return const Right<ApplicationError, List<OutlineHeading>>(
      <OutlineHeading>[],
    );
  }

  @override
  Future<Either<ApplicationError, List<LinkRef>>> backlinks(String id) async =>
      const Right<ApplicationError, List<LinkRef>>(<LinkRef>[]);

  @override
  Future<Either<ApplicationError, List<LinkRef>>> unlinkedMentions(
    String id,
  ) async => const Right<ApplicationError, List<LinkRef>>(<LinkRef>[]);
}

class _Palette implements PaletteRepository {
  _Palette(this.command) {
    latest = this;
  }

  static late _Palette latest;

  final String command;
  PaletteContext? context;

  @override
  Future<Either<ApplicationError, List<PaletteSection>>> search({
    required String query,
    required PaletteContext context,
  }) async {
    this.context = context;
    return Right<ApplicationError, List<PaletteSection>>(
      <PaletteSection>[
        PaletteSection(
          label: 'Actions',
          entries: <PaletteEntry>[
            PaletteEntry(
              target: PaletteTarget.command(id: command),
              label: command,
              icon: 'file',
              checked: false,
              depth: 0,
            ),
          ],
        ),
      ],
    );
  }
}

class _Journal extends FixtureJournalRepository {
  String? steppedOpenNote;
  bool? steppedForward;
  DayDate? openedDay;

  @override
  Future<Either<ApplicationError, MadeNote?>> step({
    required String? openNote,
    required LocalTime now,
    required bool forward,
  }) {
    steppedOpenNote = openNote;
    steppedForward = forward;
    return super.step(openNote: openNote, now: now, forward: forward);
  }

  @override
  Future<Either<ApplicationError, MadeNote>> openDay(
    DayDate day,
    LocalTime now,
  ) {
    openedDay = day;
    return super.openDay(day, now);
  }
}

class _GraphDiscovery extends FixtureDiscoveryRepository {
  @override
  Future<Either<ApplicationError, GraphLayout>> graph(
    GraphRequest request,
  ) async => const Right<ApplicationError, GraphLayout>(
    GraphLayout(nodes: <GraphNodeRow>[], edges: <GraphEdgeRow>[], linkCount: 0),
  );
}

Future<_Palette> _installPalette(String command) async {
  final palette = _Palette(command);
  if (getIt.isRegistered<PaletteBloc>()) {
    getIt.unregister<PaletteBloc>();
  }
  getIt.registerFactory<PaletteBloc>(() => PaletteBloc(palette));
  addTearDown(() {
    if (getIt.isRegistered<PaletteBloc>()) {
      getIt.unregister<PaletteBloc>();
    }
  });
  return palette;
}

Future<void> _openPalette(WidgetTester tester) async {
  final modifier = defaultTargetPlatform == TargetPlatform.iOS
      ? LogicalKeyboardKey.metaLeft
      : LogicalKeyboardKey.controlLeft;
  await tester.sendKeyDownEvent(modifier);
  await tester.sendKeyEvent(LogicalKeyboardKey.keyK);
  await tester.sendKeyUpEvent(modifier);
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 400));
}

Future<void> _run(WidgetTester tester, String command) async {
  await _openPalette(tester);
  await tester.tap(find.text(command));
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 400));
}

Future<BrowseApp> _pump(
  WidgetTester tester, {
  required String command,
  String at = _noteRoute,
  FixtureLibraryRepository? library,
  PageFixtures? fixtures,
}) async {
  await _installPalette(command);
  if (getIt.isRegistered<LibraryRepository>()) {
    getIt.unregister<LibraryRepository>();
  }
  if (getIt.isRegistered<JournalRepository>()) {
    getIt.unregister<JournalRepository>();
  }
  if (getIt.isRegistered<NoteLinksRepository>()) {
    getIt.unregister<NoteLinksRepository>();
  }
  final testLibrary = library ?? _Library();
  testLibrary.notes.putIfAbsent(_noteId, _routeNote);
  final testFixtures = fixtures ?? PageFixtures();
  final links = _Links();
  getIt
    ..registerSingleton<LibraryRepository>(testLibrary)
    ..registerSingleton<JournalRepository>(testFixtures.journal)
    ..registerSingleton<NoteLinksRepository>(links);
  addTearDown(() {
    if (getIt.isRegistered<LibraryRepository>()) {
      getIt.unregister<LibraryRepository>();
    }
    if (getIt.isRegistered<JournalRepository>()) {
      getIt.unregister<JournalRepository>();
    }
    if (getIt.isRegistered<NoteLinksRepository>()) {
      getIt.unregister<NoteLinksRepository>();
    }
  });
  return pumpBrowseApp(
    tester,
    _size,
    at: at,
    library: testLibrary,
    fixtures: testFixtures,
  );
}

void main() {
  testWidgets('pin resolves the route note without a NoteBloc above palette', (
    tester,
  ) async {
    final library = _Library()..notes[_noteId] = _routeNote();
    final app = await _pump(tester, command: 'pin', library: library);

    await _run(tester, 'pin');

    expect(library.changesMade, hasLength(1));
    expect(library.changesMade.single.$1, <String>[_noteId]);
    expect(library.changesMade.single.$2, const NoteChange.togglePin());
    expect(_Palette.latest.context?.openNoteId, _noteId);
    expect(app.where, contains('/notes/note/'));
  });

  testWidgets('outline receives the route note body', (
    tester,
  ) async {
    final library = _Library()..notes[_noteId] = _routeNote();
    await _pump(tester, command: 'show-outline', library: library);

    await _run(tester, 'show-outline');

    expect(library.noteReads, contains(_noteId));
    expect(find.byType(OutlineSheet), findsOneWidget);
    expect(_Links.latest.outlines, contains(mockOpenNote.body));
  });

  testWidgets('shows the all notes task board', (tester) async {
    final app = await _pump(tester, command: 'show-task-board');

    await _run(tester, 'show-task-board');

    expect(app.where, '/notes/board?kind=tasks');
  });

  testWidgets('shows the local graph centered on the route note', (
    tester,
  ) async {
    final app = await _pump(tester, command: 'show-local-graph');
    if (getIt.isRegistered<GraphBloc>()) {
      getIt.unregister<GraphBloc>();
    }
    getIt.registerFactory<GraphBloc>(() => GraphBloc(_GraphDiscovery()));
    addTearDown(() {
      if (getIt.isRegistered<GraphBloc>()) {
        getIt.unregister<GraphBloc>();
      }
    });

    await _run(tester, 'show-local-graph');

    expect(app.where, '/graph?center=${Uri.encodeComponent(_noteId)}&hops=2');
  });

  testWidgets('calendar opens the selected day for the route note', (
    tester,
  ) async {
    final journal = _Journal();
    await withClock(Clock.fixed(DateTime(2026, 10, 26)), () async {
      await _pump(
        tester,
        command: 'journal-calendar',
        fixtures: PageFixtures(journal: journal),
      );

      await _run(tester, 'journal-calendar');
      expect(find.byType(CalendarSheet), findsOneWidget);
      await tester.tap(
        find.bySemanticsLabel('Monday, October 12, 2026, has a note'),
      );
      await tester.pumpAndSettle();
    });

    expect(journal.openedDay, const DayDate(year: 2026, month: 10, day: 12));
    expect(journal.calls, contains('day:2026-10-12'));
  });

  for (final (command, forward) in <(String, bool)>[
    ('previous-day', false),
    ('next-day', true),
  ]) {
    testWidgets('$command steps from the route note', (tester) async {
      final journal = _Journal();
      await withClock(
        Clock.fixed(DateTime(2026, 10, 26)),
        () async {
          await _pump(
            tester,
            command: command,
            fixtures: PageFixtures(journal: journal),
          );
          await _run(tester, command);
          await tester.pumpAndSettle();
        },
      );

      expect(journal.calls, contains(forward ? 'step:forward' : 'step:back'));
      expect(journal.steppedOpenNote, _noteId);
      expect(journal.steppedForward, forward);
    });
  }

  testWidgets('share opens the route note', (tester) async {
    await _pump(tester, command: 'share');

    await _run(tester, 'share');

    expect(find.byType(ShareNoteSheet), findsOneWidget);
    expect(
      tester.widget<ShareNoteSheet>(find.byType(ShareNoteSheet)).noteId,
      _noteId,
    );
  });

  testWidgets('without an open note the palette command remains safe', (
    tester,
  ) async {
    final app = await _pump(tester, command: 'new-note', at: '/notes');

    await _run(tester, 'new-note');

    expect(_Palette.latest.context?.openNoteId, isNull);
    expect(app.where, startsWith('/notes/note/'));
  });
}
