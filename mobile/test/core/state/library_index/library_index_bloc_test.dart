// Kanso — library_index_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';

const _overview = LibraryOverview(all: 7, pinned: 2, recent: 3, trash: 1);
const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockLibraryRepository library;
  late StreamController<List<String>> changes;
  var notebooks = <NotebookRow>[];
  var tags = <TagRow>[];
  var overview = _overview;

  setUp(() {
    notebooks = <NotebookRow>[
      notebookRow('b_1', name: 'Work', count: 4),
      notebookRow('b_2', name: 'Plans', count: 2, depth: 1, parentId: 'b_1'),
    ];
    tags = <TagRow>[tagRow('t_1', name: 'todo', count: 3)];
    overview = _overview;
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.notebooks()).thenAnswer(
      (_) async => Right<ApplicationError, List<NotebookRow>>(notebooks),
    );
    when(() => library.tags()).thenAnswer(
      (_) async => Right<ApplicationError, List<TagRow>>(tags),
    );
    when(() => library.overview()).thenAnswer(
      (_) async => Right<ApplicationError, LibraryOverview>(overview),
    );
    when(() => library.overdueTasks()).thenAnswer(
      (_) async => const Right<ApplicationError, int>(2),
    );
  });

  LibraryIndexBloc build() => LibraryIndexBloc(library);

  LibraryIndexState readyState() => LibraryIndexState(
    status: LibraryIndexStatus.ready,
    notebooks: notebooks,
    tags: tags,
    overview: overview,
    overdueTasks: 2,
  );

  group('LibraryIndexBloc', () {
    test('starts loading, with nothing', () {
      final state = build().state;
      expect(state.status, LibraryIndexStatus.loading);
      expect(state.notebooks, isEmpty);
      expect(state.overview.all, 0);
    });

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'started reads the notebooks, tags and counts',
      build: build,
      act: (bloc) => bloc.add(const LibraryIndexEvent.started()),
      expect: () => <LibraryIndexState>[readyState()],
      verify: (bloc) {
        expect(bloc.state.notebook('b_2')?.depth, 1);
        expect(bloc.state.notebookName('b_1'), 'Work');
        expect(bloc.state.notebook('missing'), isNull);
        expect(bloc.state.tag('t_1')?.name, 'todo');
        expect(bloc.state.tag('missing'), isNull);
        expect(bloc.state.overview.pinned, 2);
      },
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a library with nothing in it is ready and empty',
      setUp: () {
        notebooks = <NotebookRow>[];
        tags = <TagRow>[];
        overview = const LibraryOverview(
          all: 0,
          pinned: 0,
          recent: 0,
          trash: 0,
        );
      },
      build: build,
      act: (bloc) => bloc.add(const LibraryIndexEvent.started()),
      expect: () => <LibraryIndexState>[
        const LibraryIndexState(
          status: LibraryIndexStatus.ready,
          overdueTasks: 2,
        ),
      ],
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a count of overdue tasks that fails shows none, and nothing else fails',
      setUp: () => when(() => library.overdueTasks()).thenAnswer(
        (_) async => const Left<ApplicationError, int>(_failure),
      ),
      build: build,
      act: (bloc) => bloc.add(const LibraryIndexEvent.started()),
      verify: (bloc) {
        expect(bloc.state.status, LibraryIndexStatus.ready);
        expect(bloc.state.overdueTasks, 0);
      },
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a failed reading says why',
      setUp: () => when(() => library.tags()).thenAnswer(
        (_) async => const Left<ApplicationError, List<TagRow>>(_failure),
      ),
      build: build,
      act: (bloc) => bloc.add(const LibraryIndexEvent.started()),
      expect: () => const <LibraryIndexState>[
        LibraryIndexState(status: LibraryIndexStatus.failed, error: _failure),
      ],
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'retried reads again after a failure and clears the error',
      setUp: () {
        var failing = true;
        when(() => library.overview()).thenAnswer((_) async {
          if (failing) {
            failing = false;
            return const Left<ApplicationError, LibraryOverview>(_failure);
          }
          return Right<ApplicationError, LibraryOverview>(overview);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const LibraryIndexEvent.started());
        await Future<void>.delayed(const Duration(milliseconds: 20));
        bloc.add(const LibraryIndexEvent.retried());
      },
      wait: const Duration(milliseconds: 50),
      expect: () => <Matcher>[
        isA<LibraryIndexState>()
            .having((s) => s.status, 'status', LibraryIndexStatus.failed)
            .having((s) => s.error, 'error', _failure),
        isA<LibraryIndexState>()
            .having((s) => s.status, 'status', LibraryIndexStatus.ready)
            .having((s) => s.error, 'error', isNull)
            .having((s) => s.overview.all, 'all', 7),
      ],
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a failed reading keeps what was read before',
      build: build,
      act: (bloc) async {
        bloc.add(const LibraryIndexEvent.started());
        await Future<void>.delayed(const Duration(milliseconds: 20));
        when(() => library.notebooks()).thenAnswer(
          (_) async =>
              const Left<ApplicationError, List<NotebookRow>>(_failure),
        );
        changes.add(<String>['n1']);
      },
      wait: libraryIndexDebounce * 3,
      expect: () => <LibraryIndexState>[
        readyState(),
        readyState().copyWith(
          status: LibraryIndexStatus.failed,
          error: _failure,
        ),
      ],
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a change reads the library again, without going back to loading',
      build: build,
      act: (bloc) async {
        bloc.add(const LibraryIndexEvent.started());
        await Future<void>.delayed(const Duration(milliseconds: 20));
        overview = const LibraryOverview(
          all: 8,
          pinned: 2,
          recent: 4,
          trash: 1,
        );
        notebooks = <NotebookRow>[
          ...notebooks,
          notebookRow('b_3', name: 'Home'),
        ];
        changes.add(<String>['n9']);
      },
      wait: libraryIndexDebounce * 3,
      expect: () => <Matcher>[
        isA<LibraryIndexState>().having((s) => s.overview.all, 'all', 7),
        isA<LibraryIndexState>()
            .having((s) => s.status, 'status', LibraryIndexStatus.ready)
            .having((s) => s.overview.all, 'all', 8)
            .having((s) => s.notebooks.length, 'notebooks', 3),
      ],
    );

    blocTest<LibraryIndexBloc, LibraryIndexState>(
      'a burst of changes is read once, after it stops',
      build: build,
      act: (bloc) async {
        bloc.add(const LibraryIndexEvent.started());
        await Future<void>.delayed(const Duration(milliseconds: 20));
        clearInteractions(library);
        for (var i = 0; i < 5; i++) {
          changes.add(<String>['n$i']);
          await Future<void>.delayed(const Duration(milliseconds: 20));
        }
      },
      wait: libraryIndexDebounce * 3,
      verify: (_) {
        verify(() => library.overview()).called(1);
        verify(() => library.notebooks()).called(1);
        verify(() => library.tags()).called(1);
      },
    );

    test('stops listening to the library when it is closed', () async {
      final bloc = build()..add(const LibraryIndexEvent.started());
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(changes.hasListener, isTrue);
      await bloc.close();
      expect(changes.hasListener, isFalse);
    });
  });
}
