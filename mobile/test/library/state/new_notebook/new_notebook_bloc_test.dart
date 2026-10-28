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

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(library.close);
  });

  NewNotebookBloc build() => NewNotebookBloc(library);

  group('NewNotebookBloc submitted', () {
    blocTest<NewNotebookBloc, NewNotebookState>(
      'makes the notebook at the top of the library, with the name trimmed',
      build: build,
      act: (bloc) => bloc.add(const NewNotebookEvent.submitted('  Recipes  ')),
      expect: () => <dynamic>[
        const NewNotebookState(creating: true),
        isA<NewNotebookState>()
            .having((s) => s.created?.name, 'name', 'Recipes')
            .having((s) => s.created?.parentId, 'parent', isNull),
      ],
      verify: (_) => expect(library.notebooksCreated, <(String, String?)>[
        ('Recipes', null),
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
          const Left<ApplicationError, NotebookRow>(
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
          const Left<ApplicationError, NotebookRow>(
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
