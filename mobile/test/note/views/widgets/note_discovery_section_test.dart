// Kanso — note_discovery_section_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart' hide LockState;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/note/state/note_discovery/note_discovery_bloc.dart';
import 'package:kanso/note/views/widgets/note_discovery_section.dart';

import '../../../helpers/fixture_discovery_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/pump_routed.dart';

const NoteView _note = NoteView(
  id: 'note-a',
  rev: 'revision-a',
  title: 'Garden plan',
  body: 'Local note body.',
  lock: LockState.plain(),
  notebookId: 'notebook-a',
  tags: <String>[],
  status: 'active',
  pinned: false,
  createdMs: 1,
  updatedMs: 2,
);

class _DiscoveryRepository extends FixtureDiscoveryRepository {
  int addTagCalls = 0;
  int dismissCalls = 0;
  int mergeCalls = 0;

  @override
  Future<Either<ApplicationError, bool>> relatedEnabled() async =>
      const Right<ApplicationError, bool>(true);

  @override
  Future<Either<ApplicationError, List<RelatedRow>>> relatedNotes(
    String id, {
    required int limit,
  }) async => const Right<ApplicationError, List<RelatedRow>>(
    <RelatedRow>[
      RelatedRow(
        id: 'note-b',
        title: 'Growing vegetables',
        score: 0.94,
        label: 'very close',
      ),
    ],
  );

  @override
  Future<Either<ApplicationError, NoteMeaning>> noteMeaning(String id) async =>
      const Right<ApplicationError, NoteMeaning>(
        NoteMeaning(
          tags: <SuggestedTag>[
            SuggestedTag(tagId: 'tag-ideas', name: 'ideas', weight: 0.8),
          ],
          twin: TwinRow(id: 'note-c', title: 'Garden notes', percent: 91),
        ),
      );

  @override
  Future<Either<ApplicationError, Changed>> addTag(
    String noteId,
    String tagId,
  ) async {
    addTagCalls++;
    return const Right<ApplicationError, Changed>(
      Changed(count: 1, undo: 1, message: 'Tag added.'),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> dismissTwin(
    String id,
    String otherId,
  ) async {
    dismissCalls++;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, NoteView>> mergeNotes(
    String keepId,
    String otherId,
  ) async {
    mergeCalls++;
    return const Right<ApplicationError, NoteView>(_note);
  }
}

Widget _section(_DiscoveryRepository repository) {
  return BlocProvider<NoteDiscoveryBloc>(
    create: (_) =>
        NoteDiscoveryBloc(repository)
          ..add(const NoteDiscoveryEvent.started('note-a')),
    child: const Scaffold(
      body: SingleChildScrollView(child: NoteDiscoverySection(note: _note)),
    ),
  );
}

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('related notes and meaning, ${brightness.name}', (
      tester,
    ) async {
      final repository = _DiscoveryRepository();
      await expectGoldenOnBothPlatforms(
        tester,
        _section(repository),
        name: 'phone_note_discovery_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
      expect(find.text('Growing vegetables'), findsOneWidget);
      expect(find.text('ideas'), findsOneWidget);
      expect(
        find.text('Nearly the same as Garden notes — 91%'),
        findsOneWidget,
      );
    });
  }

  testWidgets('suggested tags, dismissal and merge use their bridge actions', (
    tester,
  ) async {
    final repository = _DiscoveryRepository();
    await pumpRouted(tester, _section(repository));

    await tester.tap(find.text('ideas'));
    await tester.pumpAndSettle();
    expect(repository.addTagCalls, 1);

    await tester.tap(find.text('Dismiss'));
    await tester.pumpAndSettle();
    expect(repository.dismissCalls, 1);

    await tester.tap(find.text('Merge').first);
    await tester.pumpAndSettle();
    expect(
      find.text(
        'Garden notes will go to the trash. '
        'Its contents are kept in this note.',
      ),
      findsOneWidget,
    );
    await tester.tap(find.text('Merge').last);
    await tester.pumpAndSettle();
    expect(repository.mergeCalls, 1);
  });
}
