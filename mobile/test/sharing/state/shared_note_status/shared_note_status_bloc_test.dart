// Kanso — shared_note_status_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/state/shared_note_status/shared_note_status_bloc.dart';

import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/property.dart';
import '../../../helpers/sharing_property.dart';

const String _note = 'note:plan';

LiveStatus _status({
  ShareRole role = ShareRole.viewer,
  bool ended = false,
  List<String> problems = const <String>[],
}) => LiveStatus(
  role: role,
  connected: true,
  ended: ended,
  nearLimit: false,
  problems: problems,
);

Future<SharedNoteStatusBloc> _started(
  FixtureSharedNotesRepository shared,
) async {
  final bloc = SharedNoteStatusBloc(shared)
    ..add(const SharedNoteStatusEvent.started(_note));
  addTearDown(bloc.close);
  await bloc.stream.firstWhere((state) => state.beat > 0);
  return bloc;
}

SharedEvent _event({
  List<String> noteIds = const <String>[],
  List<String> liveNotes = const <String>[],
}) => SharedEvent(
  noteIds: noteIds,
  invitations: 0,
  commentNotes: const <String>[],
  liveNotes: liveNotes,
);

void main() {
  group('SharedNoteStatusBloc', () {
    test(
      'a note that isn’t shared has no status, and is not read only',
      () async {
        final bloc = await _started(FixtureSharedNotesRepository());
        expect(bloc.state.status, isNull);
        expect(bloc.state.readOnly, isFalse);
      },
    );

    test('reads the status of a shared note', () async {
      final shared = FixtureSharedNotesRepository()
        ..liveStatuses[_note] = _status();
      final bloc = await _started(shared);
      expect(bloc.state.status, _status());
      expect(bloc.state.readOnly, isTrue);
    });

    test('the room’s news of the note reads it again, and beats', () async {
      final shared = FixtureSharedNotesRepository()
        ..liveStatuses[_note] = _status();
      final bloc = await _started(shared);
      final beat = bloc.state.beat;
      shared.liveStatuses[_note] = _status(ended: true);
      shared.eventsController.add(_event(liveNotes: <String>[_note]));
      final now = await bloc.stream.firstWhere((s) => s.beat > beat);
      expect(now.status?.ended, isTrue);
      // A private copy is the person's to edit.
      expect(now.readOnly, isFalse);
    });

    test('news of another note is not read', () async {
      final shared = FixtureSharedNotesRepository()
        ..liveStatuses[_note] = _status();
      final bloc = await _started(shared);
      final reads = shared.calls
          .where((c) => c.startsWith('liveStatus'))
          .length;
      shared.eventsController.add(_event(liveNotes: <String>['note:other']));
      await pumpEventQueue();
      expect(
        shared.calls.where((c) => c.startsWith('liveStatus')).length,
        reads,
      );
      expect(bloc.state.beat, 1);
    });

    test('Dismiss forgets the problems', () async {
      final shared = FixtureSharedNotesRepository()
        ..liveStatuses[_note] = _status(problems: <String>['Too big.']);
      final bloc = await _started(shared);
      bloc.add(const SharedNoteStatusEvent.problemsDismissed());
      final now = await bloc.stream.firstWhere(
        (s) => s.status?.problems.isEmpty ?? false,
      );
      expect(shared.calls, contains('dismissProblems $_note'));
      expect(now.status?.role, ShareRole.viewer);
    });

    test('a status that can’t be read says why', () async {
      final shared = FixtureSharedNotesRepository()
        ..failure = const ApplicationError.network(message: 'Off.');
      final bloc = await _started(shared);
      expect(bloc.state.error, isA<NetworkError>());
      expect(bloc.state.status, isNull);
    });

    Glados<LiveStatus>(any.liveStatus, kansoExploreConfig()).test(
      'the text is read only exactly while the note is still shared to view '
      'or comment on',
      (status) async {
        final shared = FixtureSharedNotesRepository()
          ..liveStatuses[_note] = status;
        final bloc = SharedNoteStatusBloc(shared)
          ..add(const SharedNoteStatusEvent.started(_note));
        final state = await bloc.stream.firstWhere((s) => s.beat > 0);
        await bloc.close();
        final mayEdit =
            status.role == ShareRole.owner || status.role == ShareRole.editor;
        expect(state.readOnly, !status.ended && !mayEdit);
      },
    );
  });
}
