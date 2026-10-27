// Kanso — note_list_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/list_options.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/core/utils/day_grouping.dart';
import 'package:kanso/note_list/models/data/note_list_status.dart';
import 'package:kanso/note_list/state/note_list/note_list_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

const _failure = ApplicationError.storage(message: 'Disk error.');

Matcher _ready() => isA<NoteListState>().having(
  (state) => state.status,
  'status',
  NoteListStatus.ready,
);

void main() {
  late MockLibraryRepository library;
  late MockPreferencesRepository preferences;
  late MockHapticsRepository haptics;
  late MockUndoNoticeBloc undoNotice;
  late StreamController<List<String>> changes;
  var list = mockNoteList;

  setUpAll(() {
    registerFallbackValue(
      const ListQuery(
        place: ListPlace.all(),
        pinnedOnly: false,
        sort: SortKey.updated,
        descending: true,
      ),
    );
    registerFallbackValue(const NoteChange.trash());
    registerFallbackValue(const UndoNoticeEvent.dismissed());
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    list = mockNoteList;
    changes = StreamController<List<String>>.broadcast();
    addTearDown(changes.close);
    library = MockLibraryRepository();
    preferences = MockPreferencesRepository();
    haptics = MockHapticsRepository();
    undoNotice = MockUndoNoticeBloc();
    when(() => library.changes()).thenAnswer((_) => changes.stream);
    when(() => library.list(any())).thenAnswer(
      (_) async => Right<ApplicationError, NoteList>(list),
    );
    when(() => library.change(any(), any())).thenAnswer(
      (_) async => const Right<ApplicationError, Changed>(
        Changed(count: 1, undo: 7, message: 'Pinned 1 note.'),
      ),
    );
    when(() => preferences.load()).thenAnswer(
      (_) async => const Right<ApplicationError, Preferences>(Preferences()),
    );
    when(() => preferences.update(any())).thenAnswer(
      (_) async => const Right<ApplicationError, Preferences>(Preferences()),
    );
    when(() => haptics.selection()).thenAnswer((_) async {});
    when(() => haptics.impact()).thenAnswer((_) async {});
  });

  NoteListBloc build() =>
      NoteListBloc(library, preferences, haptics, undoNotice);

  group('NoteListBloc', () {
    test('starts loading, with nothing', () {
      final state = build().state;
      expect(state.status, NoteListStatus.loading);
      expect(state.rows, isEmpty);
      expect(state.selecting, isFalse);
    });

    blocTest<NoteListBloc, NoteListState>(
      'started reads the notes of the place, grouped under headers',
      build: build,
      act: (bloc) =>
          bloc.add(const NoteListEvent.started(ListPlace.notebook(id: 'b'))),
      verify: (bloc) {
        final state = bloc.state;
        expect(state.status, NoteListStatus.ready);
        expect(state.place, const ListPlace.notebook(id: 'b'));
        expect(state.rows, mockRows);
        expect(state.shape, mockShape);
        expect(state.entries, groupRows(mockRows, mockShape));
        final query =
            verify(() => library.list(captureAny())).captured.single
                as ListQuery;
        expect(query.place, const ListPlace.notebook(id: 'b'));
        expect(query.sort, SortKey.updated);
        expect(query.descending, isTrue);
        expect(query.pinnedOnly, isFalse);
        expect(query.statusOnly, isNull);
      },
    );

    blocTest<NoteListBloc, NoteListState>(
      'started goes through loading before the notes',
      build: build,
      act: (bloc) => bloc.add(const NoteListEvent.started(ListPlace.pinned())),
      expect: () => <Matcher>[
        isA<NoteListState>()
            .having((s) => s.place, 'place', const ListPlace.pinned())
            .having((s) => s.status, 'status', NoteListStatus.loading),
        _ready(),
      ],
    );

    blocTest<NoteListBloc, NoteListState>(
      'started lists in the sort the person chose before',
      setUp: () {
        when(() => preferences.load()).thenAnswer(
          (_) async => const Right<ApplicationError, Preferences>(
            Preferences(sort: SortKey.title, descending: false),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const NoteListEvent.started(ListPlace.all())),
      verify: (bloc) {
        expect(bloc.state.options.sort, SortKey.title);
        expect(bloc.state.options.descending, isFalse);
        final query =
            verify(() => library.list(captureAny())).captured.single
                as ListQuery;
        expect(query.sort, SortKey.title);
        expect(query.descending, isFalse);
      },
    );

    blocTest<NoteListBloc, NoteListState>(
      'an empty place is ready with no notes',
      setUp: () {
        list = const NoteList(rows: <NoteRow>[], shape: mockShape);
      },
      build: build,
      act: (bloc) => bloc.add(const NoteListEvent.started(ListPlace.trash())),
      verify: (bloc) {
        expect(bloc.state.status, NoteListStatus.ready);
        expect(bloc.state.rows, isEmpty);
        expect(bloc.state.entries, isEmpty);
        expect(bloc.state.inTrash, isTrue);
      },
    );

    blocTest<NoteListBloc, NoteListState>(
      'a failed reading says why, and retry reads again',
      setUp: () {
        var calls = 0;
        when(() => library.list(any())).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, NoteList>(_failure)
              : Right<ApplicationError, NoteList>(mockNoteList);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const NoteListEvent.started(ListPlace.all()));
        await bloc.stream.firstWhere(
          (state) => state.status == NoteListStatus.failed,
        );
        bloc.add(const NoteListEvent.retried());
      },
      verify: (bloc) {
        expect(bloc.state.status, NoteListStatus.ready);
        expect(bloc.state.error, isNull);
        expect(bloc.state.rows, mockRows);
      },
    );

    blocTest<NoteListBloc, NoteListState>(
      'a failed first reading is failed, with the error',
      setUp: () {
        when(() => library.list(any())).thenAnswer(
          (_) async => const Left<ApplicationError, NoteList>(_failure),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const NoteListEvent.started(ListPlace.all())),
      verify: (bloc) {
        expect(bloc.state.status, NoteListStatus.failed);
        expect(bloc.state.error, _failure);
        expect(bloc.state.rows, isEmpty);
      },
    );

    group('sort and narrowing', () {
      blocTest<NoteListBloc, NoteListState>(
        'a new sort is read and saved for every list',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc.add(
            NoteListEvent.optionsChanged(
              const ListOptions().sortedBy(SortKey.title),
            ),
          );
        },
        verify: (bloc) {
          final queries = verify(
            () => library.list(captureAny()),
          ).captured.cast<ListQuery>();
          expect(queries.last.sort, SortKey.title);
          expect(queries.last.descending, isFalse);
          final change =
              verify(() => preferences.update(captureAny())).captured.single
                  as Preferences Function(Preferences);
          final saved = change(
            const Preferences(backupNoticeSeen: true),
          );
          expect(saved.sort, SortKey.title);
          expect(saved.descending, isFalse);
          // The rest of what was saved stays.
          expect(saved.backupNoticeSeen, isTrue);
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'narrowing is read but not saved',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc.add(
            const NoteListEvent.optionsChanged(
              ListOptions(statusOnly: 'active', pinnedOnly: true),
            ),
          );
        },
        verify: (bloc) {
          final query = verify(
            () => library.list(captureAny()),
          ).captured.cast<ListQuery>().last;
          expect(query.statusOnly, 'active');
          expect(query.pinnedOnly, isTrue);
          verifyNever(() => preferences.update(any()));
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'keeps the notes in view while it reads them in the new order',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc.add(
            NoteListEvent.optionsChanged(
              const ListOptions().sortedBy(SortKey.created),
            ),
          );
        },
        skip: 2,
        expect: () => <Matcher>[
          // The options change; the notes stay until the new ones are in.
          isA<NoteListState>()
              .having((s) => s.status, 'status', NoteListStatus.ready)
              .having((s) => s.options.sort, 'sort', SortKey.created)
              .having((s) => s.rows, 'rows', mockRows),
        ],
      );
    });

    group('the library changing', () {
      blocTest<NoteListBloc, NoteListState>(
        'reads the list again, once, after a burst of writes',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          list = NoteList(rows: mockRows.sublist(1), shape: mockShape);
          changes
            ..add(<String>['a'])
            ..add(<String>['b'])
            ..add(<String>['c']);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          verify(() => library.list(any())).called(2);
          expect(bloc.state.rows, mockRows.sublist(1));
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'drops from the selection a note that is gone',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc
            ..add(const NoteListEvent.selectionStarted(MockIds.weeklyReview))
            ..add(const NoteListEvent.selectionToggled(MockIds.syncDesign));
          await Future<void>.delayed(const Duration(milliseconds: 20));
          list = NoteList(rows: mockRows.sublist(1), shape: mockShape);
          changes.add(<String>[MockIds.weeklyReview]);
        },
        wait: const Duration(milliseconds: 300),
        verify: (bloc) {
          expect(bloc.state.selected, <String>{MockIds.syncDesign});
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'a reading that a later one overtook is dropped',
        setUp: () {
          final slow = Completer<Either<ApplicationError, NoteList>>();
          var calls = 0;
          when(() => library.list(any())).thenAnswer((_) {
            calls++;
            if (calls == 2) {
              return slow.future;
            }
            if (calls == 3) {
              // The third answers first, then the second, with older data.
              Timer(const Duration(milliseconds: 30), () {
                slow.complete(
                  const Right<ApplicationError, NoteList>(
                    NoteList(rows: <NoteRow>[], shape: mockShape),
                  ),
                );
              });
            }
            return Future<Either<ApplicationError, NoteList>>.value(
              Right<ApplicationError, NoteList>(mockNoteList),
            );
          });
        },
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc
            ..add(const NoteListEvent.retried())
            ..add(const NoteListEvent.retried());
          await Future<void>.delayed(const Duration(milliseconds: 100));
        },
        verify: (bloc) {
          expect(bloc.state.rows, mockRows);
        },
      );
    });

    group('choosing several', () {
      blocTest<NoteListBloc, NoteListState>(
        'a long press starts it, and each tap chooses or lets go',
        build: build,
        seed: () => NoteListState(
          status: NoteListStatus.ready,
          rows: mockRows,
          shape: mockShape,
        ),
        act: (bloc) => bloc
          ..add(const NoteListEvent.selectionStarted(MockIds.syncDesign))
          ..add(const NoteListEvent.selectionToggled(MockIds.lisbon))
          ..add(const NoteListEvent.selectionToggled(MockIds.syncDesign)),
        expect: () => <Matcher>[
          isA<NoteListState>().having(
            (s) => s.selected,
            'selected',
            <String>{MockIds.syncDesign},
          ),
          isA<NoteListState>().having(
            (s) => s.selected,
            'selected',
            <String>{MockIds.syncDesign, MockIds.lisbon},
          ),
          isA<NoteListState>()
              .having((s) => s.selected, 'selected', <String>{MockIds.lisbon})
              .having((s) => s.selecting, 'selecting', isTrue),
        ],
        verify: (bloc) {
          verify(() => haptics.selection()).called(3);
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'letting go of the last one ends it',
        build: build,
        act: (bloc) => bloc
          ..add(const NoteListEvent.selectionStarted(MockIds.syncDesign))
          ..add(const NoteListEvent.selectionToggled(MockIds.syncDesign)),
        verify: (bloc) {
          expect(bloc.state.selecting, isFalse);
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'done clears it, with no tap of the hand',
        build: build,
        seed: () => const NoteListState(selected: <String>{'a', 'b'}),
        act: (bloc) => bloc.add(const NoteListEvent.selectionCleared()),
        expect: () => <Matcher>[
          isA<NoteListState>().having((s) => s.selected, 'selected', isEmpty),
        ],
        verify: (_) {
          verifyNever(() => haptics.selection());
        },
      );

      test('knows whether everything chosen is pinned', () {
        final state = NoteListState(
          rows: mockRows,
          selected: <String>{MockIds.weeklyReview},
        );
        expect(state.selectionPinned, isTrue);
        expect(
          state
              .copyWith(
                selected: <String>{MockIds.weeklyReview, MockIds.syncDesign},
              )
              .selectionPinned,
          isFalse,
        );
        expect(
          state.copyWith(selected: const <String>{}).selectionPinned,
          isFalse,
        );
        expect(state.selectedRows.single.id, MockIds.weeklyReview);
      });
    });

    group('changing notes', () {
      blocTest<NoteListBloc, NoteListState>(
        'pinning writes the notes once, ticks, ends choosing and reads again',
        build: build,
        act: (bloc) async {
          bloc.add(const NoteListEvent.started(ListPlace.all()));
          await bloc.stream.firstWhere(
            (state) => state.status == NoteListStatus.ready,
          );
          bloc
            ..add(const NoteListEvent.selectionStarted(MockIds.syncDesign))
            ..add(const NoteListEvent.selectionToggled(MockIds.lisbon));
          await Future<void>.delayed(Duration.zero);
          bloc.add(
            const NoteListEvent.changeRequested(
              NoteChange.togglePin(),
              <String>[MockIds.syncDesign, MockIds.lisbon],
            ),
          );
        },
        verify: (bloc) {
          final call = verify(
            () => library.change(captureAny(), captureAny()),
          ).captured;
          expect(call[0], <String>[MockIds.syncDesign, MockIds.lisbon]);
          expect(call[1], const NoteChange.togglePin());
          verify(() => library.list(any())).called(2);
          verify(() => haptics.impact()).called(1);
          expect(bloc.state.selecting, isFalse);
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'a change that wrote notes shows the notice, with Undo for it',
        build: build,
        act: (bloc) => bloc.add(
          const NoteListEvent.changeRequested(
            NoteChange.togglePin(),
            <String>[MockIds.lisbon],
          ),
        ),
        verify: (bloc) {
          verify(
            () => undoNotice.add(
              const UndoNoticeEvent.shown(7, 'Pinned 1 note.'),
            ),
          ).called(1);
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'a change that wrote nothing shows no notice',
        setUp: () {
          when(() => library.change(any(), any())).thenAnswer(
            (_) async => const Right<ApplicationError, Changed>(
              Changed(count: 0, message: ''),
            ),
          );
        },
        build: build,
        act: (bloc) => bloc.add(
          const NoteListEvent.changeRequested(
            NoteChange.trash(),
            <String>[MockIds.lisbon],
          ),
        ),
        verify: (bloc) => verifyNever(() => undoNotice.add(any())),
      );

      blocTest<NoteListBloc, NoteListState>(
        'a change that fails shows no notice',
        setUp: () {
          when(() => library.change(any(), any())).thenAnswer(
            (_) async => const Left<ApplicationError, Changed>(_failure),
          );
        },
        build: build,
        act: (bloc) => bloc.add(
          const NoteListEvent.changeRequested(
            NoteChange.trash(),
            <String>[MockIds.lisbon],
          ),
        ),
        verify: (bloc) => verifyNever(() => undoNotice.add(any())),
      );

      blocTest<NoteListBloc, NoteListState>(
        'moving and trashing do not tick the hand',
        build: build,
        act: (bloc) => bloc
          ..add(
            const NoteListEvent.changeRequested(
              NoteChange.move(notebookId: MockIds.work),
              <String>[MockIds.lisbon],
            ),
          )
          ..add(
            const NoteListEvent.changeRequested(
              NoteChange.trash(),
              <String>[MockIds.lisbon],
            ),
          ),
        verify: (bloc) {
          verify(() => library.change(any(), any())).called(2);
          verifyNever(() => haptics.impact());
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'a change that fails says why and keeps the choice',
        setUp: () {
          when(() => library.change(any(), any())).thenAnswer(
            (_) async => const Left<ApplicationError, Changed>(
              ApplicationError.locked(message: 'Locked.'),
            ),
          );
        },
        build: build,
        seed: () => const NoteListState(
          status: NoteListStatus.ready,
          selected: <String>{MockIds.lisbon},
        ),
        act: (bloc) => bloc.add(
          const NoteListEvent.changeRequested(
            NoteChange.move(notebookId: MockIds.work),
            <String>[MockIds.lisbon],
          ),
        ),
        expect: () => <Matcher>[
          isA<NoteListState>()
              .having(
                (s) => s.actionError,
                'actionError',
                const ApplicationError.locked(message: 'Locked.'),
              )
              .having((s) => s.selected, 'selected', <String>{MockIds.lisbon}),
        ],
        verify: (_) {
          verifyNever(() => haptics.impact());
          verifyNever(() => library.list(any()));
        },
      );

      blocTest<NoteListBloc, NoteListState>(
        'dismissing the error clears it',
        build: build,
        seed: () => const NoteListState(actionError: _failure),
        act: (bloc) => bloc.add(const NoteListEvent.errorDismissed()),
        expect: () => <Matcher>[
          isA<NoteListState>().having(
            (s) => s.actionError,
            'actionError',
            isNull,
          ),
        ],
      );
    });

    blocTest<NoteListBloc, NoteListState>(
      'a swipe that reaches its point gives a bump',
      build: build,
      act: (bloc) => bloc.add(const NoteListEvent.swipeArmed()),
      expect: () => <NoteListState>[],
      verify: (_) {
        verify(() => haptics.impact()).called(1);
      },
    );

    test('stops listening to the library when it is closed', () async {
      final bloc = build()..add(const NoteListEvent.started(ListPlace.all()));
      await bloc.stream.firstWhere(
        (state) => state.status == NoteListStatus.ready,
      );
      expect(changes.hasListener, isTrue);
      await bloc.close();
      expect(changes.hasListener, isFalse);
    });
  });
}
