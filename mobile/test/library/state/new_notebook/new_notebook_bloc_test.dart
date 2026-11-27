// Kanso — new_notebook_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/library/models/data/new_notebook_failure.dart';
import 'package:kanso/library/state/new_notebook/new_notebook_bloc.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(library.close);
  });

  NewNotebookBloc build() => NewNotebookBloc(library);

  NotebookRow row(String id, {String? parentId, bool locked = false}) =>
      NotebookRow(
        id: id,
        name: id,
        parentId: parentId,
        count: 0,
        locked: locked,
        depth: parentId == null ? 0 : 1,
        hasChildren: false,
      );

  group('NewNotebookBloc opened', () {
    blocTest<NewNotebookBloc, NewNotebookState>(
      'reads the starters, in the order the library has them',
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.opened()),
      expect: () => <dynamic>[
        isA<NewNotebookState>()
            .having((s) => s.starters, 'starters', mockStarters)
            .having((s) => s.parentId, 'parent', isNull)
            .having((s) => s.parentLocked, 'locked', isFalse),
      ],
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'inside an open notebook the starters stay available',
      setUp: () => library.notebookRows = <NotebookRow>[row('book:a')],
      build: build,
      act: (bloc) =>
          bloc.add(const NewNotebookEvent.opened(parentId: 'book:a')),
      verify: (bloc) {
        expect(bloc.state.parentId, 'book:a');
        expect(bloc.state.parentLocked, isFalse);
      },
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'inside a locked notebook the starters that make notes are not',
      setUp: () =>
          library.notebookRows = <NotebookRow>[row('book:a', locked: true)],
      build: build,
      act: (bloc) =>
          bloc.add(const NewNotebookEvent.opened(parentId: 'book:a')),
      verify: (bloc) => expect(bloc.state.parentLocked, isTrue),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'a notebook inside a locked one is locked too',
      setUp: () => library.notebookRows = <NotebookRow>[
        row('book:a', locked: true),
        row('book:b', parentId: 'book:a'),
      ],
      build: build,
      act: (bloc) =>
          bloc.add(const NewNotebookEvent.opened(parentId: 'book:b')),
      verify: (bloc) => expect(bloc.state.parentLocked, isTrue),
    );
  });

  group('NewNotebookBloc submitted', () {
    blocTest<NewNotebookBloc, NewNotebookState>(
      'makes the notebook at the top of the library, with the name trimmed',
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('  Recipes  ')),
      expect: () => <dynamic>[
        const NewNotebookState(creating: true),
        isA<NewNotebookState>()
            .having((s) => s.creating, 'creating', isFalse)
            .having((s) => s.created?.notebook.name, 'name', 'Recipes')
            .having((s) => s.created?.notebook.parentId, 'parent', isNull),
      ],
      verify: (_) => expect(library.notebooksCreated, <(String, String?)>[
        ('Recipes', null),
      ]),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'asks for a Video studio when that is the kind',
      build: build,
      act: (bloc) => bloc.add(
        const NewNotebookEvent.submitted(
          'Studio',
          kind: NewNotebookKind.videoStudio,
        ),
      ),
      expect: () => <dynamic>[
        const NewNotebookState(creating: true),
        isA<NewNotebookState>().having(
          (s) => s.created?.notebook.name,
          'name',
          'Studio',
        ),
      ],
      verify: (_) => expect(library.notebookKinds, <NewNotebookKind>[
        NewNotebookKind.videoStudio,
      ]),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'asks for a starter, and says which note to open',
      build: build,
      act: (bloc) => bloc.add(
        const NewNotebookEvent.submitted(
          'Kanso',
          kind: NewNotebookKind.product,
        ),
      ),
      verify: (bloc) {
        expect(library.notebookKinds, <NewNotebookKind>[
          NewNotebookKind.product,
        ]);
        expect(bloc.state.created?.firstNoteId, 'note:first');
      },
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'makes a blank notebook unless told otherwise',
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('Plain')),
      verify: (_) => expect(library.notebookKinds, <NewNotebookKind>[
        NewNotebookKind.blank,
      ]),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'makes it inside the notebook the sheet was opened from',
      setUp: () => library.notebookRows = <NotebookRow>[row('book:a')],
      build: build,
      act: (bloc) async {
        bloc.add(const NewNotebookEvent.opened(parentId: 'book:a'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(const NewNotebookEvent.submitted('Inner'));
      },
      verify: (_) => expect(library.notebooksCreated, <(String, String?)>[
        ('Inner', 'book:a'),
      ]),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'refuses a starter that makes notes inside a locked notebook',
      setUp: () =>
          library.notebookRows = <NotebookRow>[row('book:a', locked: true)],
      build: build,
      act: (bloc) async {
        bloc.add(const NewNotebookEvent.opened(parentId: 'book:a'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(
          const NewNotebookEvent.submitted(
            'Plan',
            kind: NewNotebookKind.product,
          ),
        );
      },
      verify: (bloc) {
        expect(bloc.state.failure, NewNotebookFailure.lockedParent);
        expect(library.notebooksCreated, isEmpty);
      },
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'makes a blank notebook or a studio inside a locked notebook',
      setUp: () =>
          library.notebookRows = <NotebookRow>[row('book:a', locked: true)],
      build: build,
      act: (bloc) async {
        bloc.add(const NewNotebookEvent.opened(parentId: 'book:a'));
        await Future<void>.delayed(Duration.zero);
        bloc.add(const NewNotebookEvent.submitted('Plain'));
      },
      verify: (_) => expect(library.notebooksCreated, <(String, String?)>[
        ('Plain', 'book:a'),
      ]),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'refuses an empty name without asking the library',
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('   ')),
      expect: () => const <NewNotebookState>[
        NewNotebookState(failure: NewNotebookFailure.invalidName),
      ],
      verify: (_) => expect(library.notebooksCreated, isEmpty),
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'says a name the library would not take is not one',
      setUp: () => library.createNotebookResult =
          const Left<ApplicationError, MadeNotebook>(
            ApplicationError.invalid(message: 'long'),
          ),
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('Name')),
      expect: () => const <NewNotebookState>[
        NewNotebookState(creating: true),
        NewNotebookState(failure: NewNotebookFailure.invalidName),
      ],
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'says any other failure',
      setUp: () => library.createNotebookResult =
          const Left<ApplicationError, MadeNotebook>(
            ApplicationError.storage(message: 'disk'),
          ),
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('Name')),
      expect: () => const <NewNotebookState>[
        NewNotebookState(creating: true),
        NewNotebookState(failure: NewNotebookFailure.failed),
      ],
    );

    blocTest<NewNotebookBloc, NewNotebookState>(
      'a second request while one is being made is ignored',
      setUp: () => library.notebookGate = Completer<void>(),
      build: build,
      act: (bloc) async {
        bloc
          ..add(const NewNotebookEvent.submitted('One'))
          ..add(const NewNotebookEvent.submitted('Two'));
        await Future<void>.delayed(Duration.zero);
        library.notebookGate!.complete();
      },
      verify: (_) => expect(library.notebooksCreated, hasLength(1)),
    );
  });
}
