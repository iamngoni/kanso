// Kanso — note_window_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/window/models/data/note_window_status.dart';
import 'package:kanso/window/state/note_window/note_window_bloc.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/library_fixtures.dart';

Future<void> _settle() => Future<void>.delayed(noteWindowDebounce * 3);

void main() {
  late FixtureLibraryRepository library;
  late FixtureSessionRepository session;

  setUp(() {
    library = FixtureLibraryRepository();
    session = FixtureSessionRepository();
  });

  tearDown(() async {
    await library.close();
    await session.events.close();
  });

  NoteWindowBloc make() => NoteWindowBloc(library, session);

  blocTest<NoteWindowBloc, NoteWindowState>(
    'reads the note: its title, and that it is not locked or trashed',
    build: make,
    act: (bloc) => bloc.add(const NoteWindowEvent.started(MockIds.syncDesign)),
    expect: () => const <NoteWindowState>[
      NoteWindowState(noteId: MockIds.syncDesign),
      NoteWindowState(
        noteId: MockIds.syncDesign,
        status: NoteWindowStatus.ready,
        title: 'Sync design notes',
      ),
    ],
  );

  blocTest<NoteWindowBloc, NoteWindowState>(
    'a locked note is locked, closed or open',
    build: make,
    act: (bloc) =>
        bloc.add(const NoteWindowEvent.started(MockIds.privateJournal)),
    verify: (bloc) {
      expect(bloc.state.locked, isTrue);
      expect(bloc.state.title, 'Private journal');
    },
  );

  test('a note moved to the trash elsewhere is trashed here', () async {
    final bloc = make()..add(const NoteWindowEvent.started(MockIds.syncDesign));
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);
    expect(bloc.state.trashed, isFalse);

    library.notes[MockIds.syncDesign] = mockNote(notebookId: 'trash');
    library.changeStream.add(<String>[MockIds.syncDesign]);
    await bloc.stream.firstWhere((s) => s.trashed);
    expect(bloc.state.status, NoteWindowStatus.ready);
    await bloc.close();
  });

  test('a note deleted for good is gone', () async {
    final bloc = make()..add(const NoteWindowEvent.started(MockIds.syncDesign));
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);

    library.notes.remove(MockIds.syncDesign);
    library.changeStream.add(<String>[MockIds.syncDesign]);
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.gone);
    await bloc.close();
  });

  test('a write to another note is not looked at', () async {
    final bloc = make()..add(const NoteWindowEvent.started(MockIds.syncDesign));
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);
    final states = <NoteWindowState>[];
    final subscription = bloc.stream.listen(states.add);
    library.notes.remove(MockIds.syncDesign);
    library.changeStream.add(<String>[MockIds.weeklyReview]);
    await _settle();
    expect(states, isEmpty);
    await subscription.cancel();
    await bloc.close();
  });

  test('the lock closing is looked at: the note is locked now', () async {
    final bloc = make()..add(const NoteWindowEvent.started(MockIds.syncDesign));
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);
    expect(bloc.state.locked, isFalse);

    library.notes[MockIds.syncDesign] = mockNote(closed: true);
    session.events.add(const LockEvent.locked(reason: LockReason.manual));
    await bloc.stream.firstWhere((s) => s.locked);
    await bloc.close();
  });

  test('a failure other than not found leaves the window as it was', () async {
    final bloc = make()..add(const NoteWindowEvent.started(MockIds.syncDesign));
    await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);
    library.noteFailure = const ApplicationError.storage(message: 'disk');
    library.changeStream.add(<String>[MockIds.syncDesign]);
    await _settle();
    expect(bloc.state.status, NoteWindowStatus.ready);
    await bloc.close();
  });

  test(
    'restoring moves the note to the first notebook that is not the trash',
    () async {
      final bloc = make()
        ..add(const NoteWindowEvent.started(MockIds.syncDesign));
      await bloc.stream.firstWhere((s) => s.status == NoteWindowStatus.ready);
      bloc.add(const NoteWindowEvent.restoreRequested());
      await Future<void>.delayed(const Duration(milliseconds: 50));
      expect(library.changesMade, hasLength(1));
      final (ids, change) = library.changesMade.single;
      expect(ids, <String>[MockIds.syncDesign]);
      expect(change, isA<NoteChange_Move>());
      expect((change as NoteChange_Move).notebookId, isNot('trash'));
      await bloc.close();
    },
  );
}
