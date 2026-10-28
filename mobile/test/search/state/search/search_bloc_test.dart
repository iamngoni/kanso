// Kanso — search_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/search_scope_kind.dart';
import 'package:kanso/search/models/data/search_status.dart';
import 'package:kanso/search/state/search/search_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

void main() {
  late MockLibraryRepository library;
  late MockDiscoveryRepository discovery;
  late StreamController<List<String>> changes;
  var hits = mockHits;

  setUpAll(() {
    registerFallbackValue(const SearchScope.all());
  });

  setUp(() {
    hits = mockHits;
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    discovery = MockDiscoveryRepository();
    when(() => discovery.meaningQuery(any())).thenReturn(null);
    when(() => discovery.relatedEnabled()).thenAnswer(
      (_) async => const Right<ApplicationError, bool>(false),
    );
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(
      () => library.search(
        any(),
        scope: any(named: 'scope'),
        limit: any(named: 'limit'),
      ),
    ).thenAnswer((_) async => Right<ApplicationError, List<Hit>>(hits));
  });

  SearchBloc build() => SearchBloc(library, discovery);

  List<(String, SearchScope)> searched() {
    final calls = verify(
      () => library.search(
        captureAny(),
        scope: captureAny(named: 'scope'),
        limit: any(named: 'limit'),
      ),
    ).captured;
    return <(String, SearchScope)>[
      for (var i = 0; i < calls.length; i += 2)
        (calls[i] as String, calls[i + 1] as SearchScope),
    ];
  }

  group('SearchBloc', () {
    blocTest<SearchBloc, SearchState>(
      'shows local semantic matches only when the model is enabled',
      setUp: () {
        when(
          () => discovery.meaningQuery('orchard fruit'),
        ).thenReturn('orchard fruit');
        when(() => discovery.relatedEnabled()).thenAnswer(
          (_) async => const Right<ApplicationError, bool>(true),
        );
        when(
          () => discovery.searchByMeaning('orchard fruit', k: 20),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, MeaningResult>(
            MeaningResult.found(
              rows: <RelatedRow>[
                RelatedRow(
                  id: 'semantic-note',
                  title: 'Fruit garden',
                  score: 0.9,
                  label: 'very close',
                ),
              ],
            ),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(
        const SearchEvent.started('orchard fruit', SearchScopeKind.all, null),
      ),
      wait: const Duration(milliseconds: 20),
      expect: () => <Matcher>[
        isA<SearchState>().having(
          (state) => state.status,
          'status',
          SearchStatus.idle,
        ),
        isA<SearchState>().having(
          (state) => state.status,
          'status',
          SearchStatus.searching,
        ),
        isA<SearchState>().having(
          (state) => state.meaningHits,
          'meaning matches',
          <RelatedRow>[
            const RelatedRow(
              id: 'semantic-note',
              title: 'Fruit garden',
              score: 0.9,
              label: 'very close',
            ),
          ],
        ),
      ],
      verify: (_) {
        verify(
          () => discovery.searchByMeaning('orchard fruit', k: 20),
        ).called(1);
      },
    );

    test('starts idle, with nothing to look for', () {
      final state = build().state;
      expect(state.status, SearchStatus.idle);
      expect(state.hasQuery, isFalse);
      expect(state.hits, isEmpty);
    });

    blocTest<SearchBloc, SearchState>(
      'started with words searches at once, in the scope it was opened with',
      build: build,
      act: (bloc) => bloc.add(
        const SearchEvent.started('sync', SearchScopeKind.titles, null),
      ),
      expect: () => <Matcher>[
        isA<SearchState>()
            .having((s) => s.query, 'query', 'sync')
            .having((s) => s.status, 'status', SearchStatus.idle),
        isA<SearchState>().having(
          (s) => s.status,
          'status',
          SearchStatus.searching,
        ),
        isA<SearchState>()
            .having((s) => s.status, 'status', SearchStatus.ready)
            .having((s) => s.hits, 'hits', mockHits),
      ],
      verify: (_) {
        verify(
          () => library.search(
            'sync',
            scope: const SearchScope.titles(),
          ),
        ).called(1);
      },
    );

    blocTest<SearchBloc, SearchState>(
      'started with nothing typed searches nothing',
      build: build,
      act: (bloc) => bloc.add(
        const SearchEvent.started('', SearchScopeKind.all, null),
      ),
      verify: (bloc) {
        expect(bloc.state.status, SearchStatus.idle);
        verifyNever(
          () => library.search(
            any(),
            scope: any(named: 'scope'),
            limit: any(named: 'limit'),
          ),
        );
      },
    );

    blocTest<SearchBloc, SearchState>(
      'a query of spaces is nothing to look for',
      build: build,
      act: (bloc) => bloc
        ..add(const SearchEvent.started('', SearchScopeKind.all, null))
        ..add(const SearchEvent.queryChanged('   ')),
      wait: const Duration(milliseconds: 300),
      verify: (bloc) {
        expect(bloc.state.status, SearchStatus.idle);
        verifyNever(
          () => library.search(
            any(),
            scope: any(named: 'scope'),
            limit: any(named: 'limit'),
          ),
        );
      },
    );

    group('typing', () {
      blocTest<SearchBloc, SearchState>(
        'searches once, for the last words, when the person pauses',
        build: build,
        act: (bloc) async {
          bloc.add(const SearchEvent.started('', SearchScopeKind.all, null));
          await Future<void>.delayed(Duration.zero);
          bloc
            ..add(const SearchEvent.queryChanged('s'))
            ..add(const SearchEvent.queryChanged('sy'))
            ..add(const SearchEvent.queryChanged('syn'));
        },
        wait: const Duration(milliseconds: 400),
        verify: (bloc) {
          expect(searched(), <(String, SearchScope)>[
            ('syn', const SearchScope.all()),
          ]);
          expect(bloc.state.hits, mockHits);
        },
      );

      blocTest<SearchBloc, SearchState>(
        'does not search for words taken back before the pause ends',
        build: build,
        act: (bloc) async {
          bloc.add(const SearchEvent.started('', SearchScopeKind.all, null));
          await Future<void>.delayed(Duration.zero);
          bloc
            ..add(const SearchEvent.queryChanged('abc'))
            ..add(const SearchEvent.cleared());
        },
        wait: const Duration(milliseconds: 400),
        verify: (bloc) {
          expect(bloc.state.status, SearchStatus.idle);
          expect(bloc.state.query, '');
          verifyNever(
            () => library.search(
              any(),
              scope: any(named: 'scope'),
              limit: any(named: 'limit'),
            ),
          );
        },
      );

      blocTest<SearchBloc, SearchState>(
        'keeps the results in view while it looks for the next words',
        build: build,
        act: (bloc) async {
          bloc.add(
            const SearchEvent.started('sync', SearchScopeKind.all, null),
          );
          await bloc.stream.firstWhere(
            (state) => state.status == SearchStatus.ready,
          );
          bloc.add(const SearchEvent.queryChanged('syncing'));
        },
        wait: const Duration(milliseconds: 400),
        skip: 3,
        // The words change with the old results still there; the new ones
        // are the same here, so nothing more is said.
        expect: () => <Matcher>[
          isA<SearchState>()
              .having((s) => s.query, 'query', 'syncing')
              .having((s) => s.status, 'status', SearchStatus.ready)
              .having((s) => s.hits, 'hits', mockHits),
        ],
      );
    });

    group('where to look', () {
      blocTest<SearchBloc, SearchState>(
        'a new scope searches the same words there, at once',
        build: build,
        act: (bloc) async {
          bloc.add(
            const SearchEvent.started('sync', SearchScopeKind.all, null),
          );
          await bloc.stream.firstWhere(
            (state) => state.status == SearchStatus.ready,
          );
          bloc
            ..add(const SearchEvent.scopeChanged(SearchScopeKind.tags))
            ..add(const SearchEvent.scopeChanged(SearchScopeKind.titles));
        },
        verify: (bloc) {
          expect(bloc.state.scope, SearchScopeKind.titles);
          final scopes = searched().map((call) => call.$2).toList();
          expect(scopes.first, const SearchScope.all());
          expect(scopes.last, const SearchScope.titles());
        },
      );

      blocTest<SearchBloc, SearchState>(
        'the notebook scope looks in the notebook the search was opened from',
        build: build,
        act: (bloc) => bloc.add(
          const SearchEvent.started(
            'sync',
            SearchScopeKind.notebook,
            MockIds.work,
          ),
        ),
        verify: (bloc) {
          expect(
            searched().single.$2,
            const SearchScope.notebook(id: MockIds.work),
          );
          expect(bloc.state.notebookId, MockIds.work);
        },
      );

      blocTest<SearchBloc, SearchState>(
        'the notebook scope with no notebook looks everywhere',
        build: build,
        act: (bloc) => bloc.add(
          const SearchEvent.started('sync', SearchScopeKind.notebook, null),
        ),
        verify: (bloc) {
          expect(searched().single.$2, const SearchScope.all());
        },
      );
    });

    blocTest<SearchBloc, SearchState>(
      'finding nothing is ready with no hits',
      setUp: () {
        hits = const <Hit>[];
      },
      build: build,
      act: (bloc) => bloc.add(
        const SearchEvent.started('zzz', SearchScopeKind.all, null),
      ),
      verify: (bloc) {
        expect(bloc.state.status, SearchStatus.ready);
        expect(bloc.state.hits, isEmpty);
      },
    );

    blocTest<SearchBloc, SearchState>(
      'a failed search says why, and retry searches again',
      setUp: () {
        var calls = 0;
        when(
          () => library.search(
            any(),
            scope: any(named: 'scope'),
            limit: any(named: 'limit'),
          ),
        ).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, List<Hit>>(_failure)
              : Right<ApplicationError, List<Hit>>(mockHits);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const SearchEvent.started('sync', SearchScopeKind.all, null));
        await bloc.stream.firstWhere(
          (state) => state.status == SearchStatus.failed,
        );
        expect(bloc.state.error, _failure);
        bloc.add(const SearchEvent.retried());
      },
      verify: (bloc) {
        expect(bloc.state.status, SearchStatus.ready);
        expect(bloc.state.error, isNull);
      },
    );

    group('the library changing', () {
      blocTest<SearchBloc, SearchState>(
        'searches again, once, after a burst of writes',
        build: build,
        act: (bloc) async {
          bloc.add(
            const SearchEvent.started('sync', SearchScopeKind.all, null),
          );
          await bloc.stream.firstWhere(
            (state) => state.status == SearchStatus.ready,
          );
          hits = mockHits.sublist(1);
          changes
            ..add(<String>['a'])
            ..add(<String>['b']);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          expect(searched(), hasLength(2));
          expect(bloc.state.hits, mockHits.sublist(1));
        },
      );

      blocTest<SearchBloc, SearchState>(
        'has nothing to search again when nothing is typed',
        build: build,
        act: (bloc) async {
          bloc.add(const SearchEvent.started('', SearchScopeKind.all, null));
          await Future<void>.delayed(Duration.zero);
          changes.add(<String>['a']);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          expect(bloc.state.status, SearchStatus.idle);
          verifyNever(
            () => library.search(
              any(),
              scope: any(named: 'scope'),
              limit: any(named: 'limit'),
            ),
          );
        },
      );
    });

    blocTest<SearchBloc, SearchState>(
      'a search that a later one overtook is dropped',
      setUp: () {
        final slow = Completer<Either<ApplicationError, List<Hit>>>();
        when(
          () => library.search(
            'slow',
            scope: any(named: 'scope'),
            limit: any(named: 'limit'),
          ),
        ).thenAnswer((_) => slow.future);
        when(
          () => library.search(
            'fast',
            scope: any(named: 'scope'),
            limit: any(named: 'limit'),
          ),
        ).thenAnswer((_) async {
          Timer(const Duration(milliseconds: 20), () {
            slow.complete(Right<ApplicationError, List<Hit>>(mockHits));
          });
          return const Right<ApplicationError, List<Hit>>(<Hit>[]);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const SearchEvent.started('slow', SearchScopeKind.all, null));
        await Future<void>.delayed(const Duration(milliseconds: 10));
        bloc.add(const SearchEvent.started('fast', SearchScopeKind.all, null));
        await Future<void>.delayed(const Duration(milliseconds: 100));
      },
      verify: (bloc) {
        expect(bloc.state.query, 'fast');
        expect(bloc.state.hits, isEmpty);
      },
    );

    test('stops listening to the library when it is closed', () async {
      final bloc = build()
        ..add(const SearchEvent.started('sync', SearchScopeKind.all, null));
      await bloc.stream.firstWhere(
        (state) => state.status == SearchStatus.ready,
      );
      expect(changes.hasListener, isTrue);
      await bloc.close();
      expect(changes.hasListener, isFalse);
    });
  });
}
