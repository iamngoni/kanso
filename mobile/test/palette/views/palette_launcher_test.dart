// Kanso — palette_launcher_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' hide LockState;
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/assist_action.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/repos/abs/journal_repository.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/models/repos/abs/templates_repository.dart';
import 'package:kanso/discovery/models/repos/abs/discovery_repository.dart';
import 'package:kanso/graph/state/graph/graph_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/journal/views/widgets/calendar_sheet.dart';
import 'package:kanso/lock/models/data/lock_target.dart';
import 'package:kanso/lock/views/widgets/change_password_sheet.dart';
import 'package:kanso/lock/views/widgets/lock_item_sheet.dart';
import 'package:kanso/lock/views/widgets/remove_lock_sheet.dart';
import 'package:kanso/note/models/data/note_mode.dart';
import 'package:kanso/note/state/note/note_bloc.dart';
import 'package:kanso/note/state/note_assist_intent/note_assist_intent_bloc.dart';
import 'package:kanso/note/state/note_mode_intent/note_mode_intent_bloc.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/note/views/widgets/related_notes_sheet.dart';
import 'package:kanso/note_links/models/repos/abs/note_links_repository.dart';
import 'package:kanso/note_links/views/widgets/outline_sheet.dart';
import 'package:kanso/palette/models/repos/abs/palette_repository.dart';
import 'package:kanso/palette/state/palette/palette_bloc.dart';
import 'package:kanso/sharing/views/widgets/share_note_sheet.dart';
import 'package:kanso/templates/state/template_picker/template_picker_bloc.dart';
import 'package:kanso/templates/views/widgets/template_picker_sheet.dart';

import '../../helpers/ai_blocs.dart';
import '../../helpers/fixture_ai_repository.dart';
import '../../helpers/fixture_discovery_repository.dart';
import '../../helpers/fixture_journal_repository.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/fixture_session_repository.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';
import '../../helpers/pump_routed.dart';

const Size _size = Size(1180, 820);
const String _noteId = 'note:palette-test';
const String _noteRoute = '/notes/note/note%3Apalette-test';

NoteView _routeNote({
  String id = _noteId,
  LockState lock = const LockState.plain(),
}) => NoteView(
  id: id,
  rev: mockOpenNote.rev,
  title: mockOpenNote.title,
  body: mockOpenNote.body,
  lock: lock,
  notebookId: mockOpenNote.notebookId,
  tags: mockOpenNote.tags,
  status: mockOpenNote.status,
  pinned: mockOpenNote.pinned,
  createdMs: mockOpenNote.createdMs,
  updatedMs: mockOpenNote.updatedMs,
);

class _Library extends FixtureLibraryRepository {
  final List<String> noteReads = <String>[];
  Completer<void>? createGate;

  @override
  Future<Either<ApplicationError, NoteView>> note(String id) {
    noteReads.add(id);
    return super.note(id);
  }

  @override
  Future<Either<ApplicationError, Saved>> create(NewNote note) async {
    await createGate?.future;
    return super.create(note);
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
    if ((command == 'edit-mode' || command == 'preview-mode') &&
        context.openNoteId == null) {
      return const Right<ApplicationError, List<PaletteSection>>(
        <PaletteSection>[],
      );
    }
    const aiNoteCommands = <String>{
      'summarize-note',
      'improve-writing',
      'make-shorter',
      'fix-spelling-and-grammar',
      'continue-writing',
      'summarize-selection',
    };
    if ((aiNoteCommands.contains(command) &&
            (!context.aiOn || context.openNoteId == null)) ||
        (command == 'ask-your-notes' && !context.aiOn)) {
      return const Right<ApplicationError, List<PaletteSection>>(
        <PaletteSection>[],
      );
    }
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

class _Templates implements TemplatesRepository {
  final List<TemplateRow> rows = <TemplateRow>[
    const TemplateRow(id: 'template:chosen', title: 'Chosen template'),
  ];
  final MadeNote made = const MadeNote(id: 'note:made', titleFirst: false);
  String? pickedId;
  String? pickedBookId;
  LocalTime? pickedNow;
  int newTemplateCalls = 0;

  @override
  Future<Either<ApplicationError, List<TemplateRow>>> templates() async {
    return Right<ApplicationError, List<TemplateRow>>(rows);
  }

  @override
  Future<Either<ApplicationError, MadeNote>> newFromTemplate({
    required String templateId,
    required String? bookId,
    required LocalTime now,
  }) async {
    pickedId = templateId;
    pickedBookId = bookId;
    pickedNow = now;
    return Right<ApplicationError, MadeNote>(made);
  }

  @override
  Future<Either<ApplicationError, MadeNote>> newTemplate() async {
    newTemplateCalls++;
    return Right<ApplicationError, MadeNote>(made);
  }

  @override
  Future<Either<ApplicationError, String?>> dailyTemplate() async =>
      const Right<ApplicationError, String?>(null);

  @override
  Future<Either<ApplicationError, Unit>> setDailyTemplate(
    String? templateId,
  ) async => const Right<ApplicationError, Unit>(unit);
}

class _RelatedDiscovery extends FixtureDiscoveryRepository {
  _RelatedDiscovery({required this.enabled, this.rows = const <RelatedRow>[]});

  final bool enabled;
  final List<RelatedRow> rows;
  String? requestedNoteId;
  int? requestedLimit;

  @override
  Future<Either<ApplicationError, bool>> relatedEnabled() async =>
      Right<ApplicationError, bool>(enabled);

  @override
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  }) async {
    requestedNoteId = id;
    requestedLimit = limit;
    return Right<ApplicationError, List<RelatedRow>>(rows);
  }
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

void _requestMode(
  NoteModeIntentBloc intents,
  String noteId,
  NoteMode mode,
) {
  intents.add(NoteModeIntentEvent.requested(noteId: noteId, mode: mode));
}

Future<BrowseApp> _pump(
  WidgetTester tester, {
  required String command,
  String at = _noteRoute,
  FixtureLibraryRepository? library,
  PageFixtures? fixtures,
  FixtureSessionRepository? session,
  _Templates? templates,
  _RelatedDiscovery? discovery,
  FixtureAiRepository? ai,
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
  if (getIt.isRegistered<TemplatesRepository>()) {
    getIt.unregister<TemplatesRepository>();
  }
  if (getIt.isRegistered<DiscoveryRepository>()) {
    getIt.unregister<DiscoveryRepository>();
  }
  final testLibrary = library ?? _Library();
  testLibrary.notes.putIfAbsent(_noteId, _routeNote);
  final testFixtures = fixtures ?? PageFixtures();
  final links = _Links();
  final testTemplates = templates ?? _Templates();
  final testDiscovery = discovery ?? _RelatedDiscovery(enabled: false);
  getIt
    ..registerSingleton<LibraryRepository>(testLibrary)
    ..registerSingleton<JournalRepository>(testFixtures.journal)
    ..registerSingleton<NoteLinksRepository>(links)
    ..registerSingleton<TemplatesRepository>(testTemplates)
    ..registerSingleton<DiscoveryRepository>(testDiscovery);
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
    if (getIt.isRegistered<TemplatesRepository>()) {
      getIt.unregister<TemplatesRepository>();
    }
    if (getIt.isRegistered<DiscoveryRepository>()) {
      getIt.unregister<DiscoveryRepository>();
    }
    if (getIt.isRegistered<TemplatePickerBloc>()) {
      getIt.unregister<TemplatePickerBloc>();
    }
  });
  final app = await pumpBrowseApp(
    tester,
    _size,
    at: at,
    library: testLibrary,
    session: session,
    fixtures: testFixtures,
    ai: ai,
  );
  if (getIt.isRegistered<TemplatePickerBloc>()) {
    getIt.unregister<TemplatePickerBloc>();
  }
  getIt.registerFactory<TemplatePickerBloc>(
    () => TemplatePickerBloc(testTemplates),
  );
  addTearDown(() {
    if (getIt.isRegistered<TemplatePickerBloc>()) {
      getIt.unregister<TemplatePickerBloc>();
    }
  });
  return app;
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

  testWidgets('lock opens the lock sheet for a plain route note', (
    tester,
  ) async {
    await _pump(tester, command: 'lock');

    await _run(tester, 'lock');

    expect(find.byType(LockItemSheet), findsOneWidget);
    expect(
      tester.widget<LockItemSheet>(find.byType(LockItemSheet)).target,
      const LockTarget.note(_noteId),
    );
  });

  for (final lock in <LockState>[
    const LockState.open(),
    const LockState.closed(),
  ]) {
    testWidgets('lock does not open for $lock', (tester) async {
      final library = _Library()..notes[_noteId] = _routeNote(lock: lock);
      await _pump(tester, command: 'lock', library: library);

      await _run(tester, 'lock');

      expect(find.byType(LockItemSheet), findsNothing);
    });
  }

  testWidgets('remove-lock opens for an open locked route note', (
    tester,
  ) async {
    final session = FixtureSessionRepository(unlocked: true);
    final library = _Library()
      ..notes[_noteId] = _routeNote(lock: const LockState.open());
    await _pump(
      tester,
      command: 'remove-lock',
      library: library,
      session: session,
    );

    await _run(tester, 'remove-lock');

    expect(find.byType(RemoveLockSheet), findsOneWidget);
    expect(
      tester.widget<RemoveLockSheet>(find.byType(RemoveLockSheet)).target,
      const LockTarget.note(_noteId),
    );
  });

  testWidgets('lock-now locks the open session', (tester) async {
    final session = FixtureSessionRepository(unlocked: true);
    await _pump(
      tester,
      command: 'lock-now',
      at: '/notes',
      session: session,
    );

    await _run(tester, 'lock-now');

    expect(session.calls, contains('lock'));
  });

  testWidgets('lock-password opens the change password sheet', (tester) async {
    await _pump(tester, command: 'lock-password', at: '/notes');

    await _run(tester, 'lock-password');

    expect(find.byType(ChangePasswordSheet), findsOneWidget);
  });

  testWidgets('new-from-template creates and opens the chosen template', (
    tester,
  ) async {
    final templates = _Templates();
    final app = await _pump(
      tester,
      command: 'new-from-template',
      at: const BrowseLocation.place(
        ListPlace.notebook(id: 'book:palette'),
      ).location,
      templates: templates,
    );

    await _run(tester, 'new-from-template');
    expect(find.byType(TemplatePickerSheet), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 500));
    await tester.tap(find.text('Chosen template'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(templates.pickedId, 'template:chosen');
    expect(templates.pickedBookId, 'book:palette');
    expect(templates.pickedNow, isNotNull);
    expect(app.where, contains('note%3Amade'));
  });

  testWidgets('new-template creates and opens a template note', (
    tester,
  ) async {
    final templates = _Templates();
    final app = await _pump(
      tester,
      command: 'new-template',
      at: '/notes',
      templates: templates,
    );

    await _run(tester, 'new-template');

    expect(templates.newTemplateCalls, 1);
    expect(app.where, contains('note%3Amade'));
  });

  testWidgets('voice-input opens the new note route with voice enabled', (
    tester,
  ) async {
    final library = _Library()..createGate = Completer<void>();
    final app = await _pump(
      tester,
      command: 'voice-input',
      at: '/notes',
      library: library,
    );

    await _openPalette(tester);
    await tester.tap(find.text('voice-input'));
    await tester.pump();

    expect(app.where, '/new?voice=1');
    library.createGate!.complete();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
  });

  testWidgets('show-related lists and opens a related route note', (
    tester,
  ) async {
    final discovery = _RelatedDiscovery(
      enabled: true,
      rows: const <RelatedRow>[
        RelatedRow(
          id: 'note:related',
          title: 'Related note',
          score: 0.9,
          label: 'Garden',
        ),
      ],
    );
    final app = await _pump(
      tester,
      command: 'show-related',
      discovery: discovery,
    );

    await _run(tester, 'show-related');
    expect(find.byType(RelatedNotesSheet), findsOneWidget);
    expect(find.text('Related note'), findsOneWidget);
    expect(discovery.requestedNoteId, _noteId);
    expect(discovery.requestedLimit, 20);
    await tester.tap(find.text('Related note'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));

    expect(app.where, contains('note%3Arelated'));
    expect(find.byType(RelatedNotesSheet), findsNothing);
  });

  testWidgets('show-related offers turn-on copy while related notes are off', (
    tester,
  ) async {
    final discovery = _RelatedDiscovery(enabled: false);
    await _pump(
      tester,
      command: 'show-related',
      discovery: discovery,
    );

    await _run(tester, 'show-related');

    expect(
      find.text(
        'Turn on Related notes in Settings to see notes about the same things.',
      ),
      findsOneWidget,
    );
    expect(find.byType(RelatedNotesSheet), findsOneWidget);
  });

  testWidgets('without an open note the palette command remains safe', (
    tester,
  ) async {
    final app = await _pump(tester, command: 'new-note', at: '/notes');

    await _run(tester, 'new-note');

    expect(_Palette.latest.context?.openNoteId, isNull);
    expect(app.where, startsWith('/notes/note/'));
  });

  testWidgets('edit-mode switches the open note without replacing its bloc', (
    tester,
  ) async {
    await _pump(tester, command: 'edit-mode');
    final note = tester.element(find.byType(NotePane)).read<NoteBloc>();

    expect(note.state.mode, NoteMode.preview);
    await _run(tester, 'edit-mode');

    expect(note.state.mode, NoteMode.edit);
    expect(tester.element(find.byType(NotePane)).read<NoteBloc>(), same(note));
  });

  testWidgets('preview-mode returns the open note to preview', (tester) async {
    await _pump(tester, command: 'preview-mode');
    final noteElement = tester.element(find.byType(NotePane));
    final note = BlocProvider.of<NoteBloc>(noteElement);
    final intents = BlocProvider.of<NoteModeIntentBloc>(noteElement);
    _requestMode(intents, _noteId, NoteMode.edit);
    await tester.pump();
    expect(note.state.mode, NoteMode.edit);

    await _run(tester, 'preview-mode');

    expect(note.state.mode, NoteMode.preview);
    expect(tester.element(find.byType(NotePane)).read<NoteBloc>(), same(note));
  });

  for (final command in <String>['edit-mode', 'preview-mode']) {
    testWidgets('$command is absent without an open note', (tester) async {
      await _pump(tester, command: command, at: '/notes');

      await _openPalette(tester);

      expect(_Palette.latest.context?.openNoteId, isNull);
      expect(find.text(command), findsNothing);
    });
  }

  testWidgets('a mode request for note A does not change note B', (
    tester,
  ) async {
    const noteB = 'note:other';
    final library = _Library()..notes[noteB] = _routeNote(id: noteB);
    await _pump(
      tester,
      command: 'edit-mode',
      at: '/notes/note/note%3Aother',
      library: library,
    );
    final noteElement = tester.element(find.byType(NotePane));
    final note = BlocProvider.of<NoteBloc>(noteElement);
    final intents = BlocProvider.of<NoteModeIntentBloc>(noteElement);

    _requestMode(intents, _noteId, NoteMode.edit);
    await tester.pump();

    expect(note.state.noteId, noteB);
    expect(note.state.mode, NoteMode.preview);
  });

  group('AI palette commands', () {
    testWidgets('palette reads live AI state and hides AI commands while off', (
      tester,
    ) async {
      await _pump(tester, command: 'improve-writing');

      await _openPalette(tester);

      expect(_Palette.latest.context?.aiOn, isFalse);
      expect(find.text('improve-writing'), findsNothing);
    });

    testWidgets('AI commands appear when AI is enabled', (tester) async {
      final ai = FixtureAiRepository(
        settings: aiSettings(enabled: true, hasKey: true),
      );
      await _pump(tester, command: 'improve-writing', ai: ai);

      await _openPalette(tester);

      expect(_Palette.latest.context?.aiOn, isTrue);
      expect(find.text('improve-writing'), findsOneWidget);
    });

    testWidgets('Ask Your Notes opens the Ask screen', (tester) async {
      final ai = FixtureAiRepository(
        settings: aiSettings(enabled: true, hasKey: true),
      );
      final app = await _pump(
        tester,
        command: 'ask-your-notes',
        at: '/notes',
        ai: ai,
      );

      await _run(tester, 'ask-your-notes');

      expect(app.where, '/ask');
    });

    testWidgets('AI settings opens the AI settings screen', (tester) async {
      final app = await _pump(
        tester,
        command: 'settings-ai',
        at: '/notes',
      );

      await _run(tester, 'settings-ai');

      expect(app.where, '/settings/ai');
    });

    testWidgets('Join a shared note opens the join screen', (tester) async {
      final app = await _pump(
        tester,
        command: 'join-shared-note',
        at: '/notes',
      );

      await _run(tester, 'join-shared-note');

      expect(topOf(app.router), '/shared/join');
    });

    testWidgets('Summarize Note stays in Preview and opens the summary', (
      tester,
    ) async {
      final ai = FixtureAiRepository(
        settings: aiSettings(enabled: true, hasKey: true),
      );
      await _pump(tester, command: 'summarize-note', ai: ai);
      final noteElement = tester.element(find.byType(NotePane));
      final note = BlocProvider.of<NoteBloc>(noteElement);
      final intents = BlocProvider.of<NoteAssistIntentBloc>(noteElement);

      await _run(tester, 'summarize-note');
      await tester.pumpAndSettle();

      expect(note.state.mode, NoteMode.preview);
      expect(find.text('Summary'), findsOneWidget);
      expect(ai.summarized, <String>[_noteId]);
      expect(intents.state.pending, isFalse);
    });

    testWidgets('An inline action enters Edit and consumes the request', (
      tester,
    ) async {
      final ai = FixtureAiRepository(
        settings: aiSettings(enabled: true, hasKey: true),
      );
      await _pump(tester, command: 'improve-writing', ai: ai);
      final noteElement = tester.element(find.byType(NotePane));
      final note = BlocProvider.of<NoteBloc>(noteElement);
      final intents = BlocProvider.of<NoteAssistIntentBloc>(noteElement);

      await _run(tester, 'improve-writing');
      await tester.pumpAndSettle();

      expect(note.state.mode, NoteMode.edit);
      // Two panes are mounted in this layout, and each Scaffold shows the
      // same snackbar from the shared messenger.
      expect(find.text('Select some text first.'), findsWidgets);
      expect(intents.state.pending, isFalse);
    });

    testWidgets('An assistant request for note A does not change note B', (
      tester,
    ) async {
      const noteB = 'note:other';
      final library = _Library()..notes[noteB] = _routeNote(id: noteB);
      await _pump(
        tester,
        command: 'improve-writing',
        at: '/notes/note/note%3Aother',
        library: library,
      );
      final noteElement = tester.element(find.byType(NotePane));
      final intents = BlocProvider.of<NoteAssistIntentBloc>(noteElement)
        ..add(
          const NoteAssistIntentEvent.requested(
            noteId: _noteId,
            action: AssistAction.improve,
          ),
        );
      await tester.pump();

      final note = tester.element(find.byType(Scaffold).last).read<NoteBloc>();
      expect(note.state.noteId, noteB);
      expect(note.state.mode, NoteMode.preview);
      expect(intents.state.pending, isTrue);
    });
  });
}
