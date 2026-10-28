// Kanso — navigation_history_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/navigation_history.dart';
import 'package:kanso/core/state/navigation_history/navigation_history_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';

void main() {
  late MockLibraryRepository library;
  final trashed = <String>{};
  final deleted = <String>{};

  setUp(() {
    trashed.clear();
    deleted.clear();
    library = MockLibraryRepository();
    when(() => library.note(any())).thenAnswer((call) async {
      final id = call.positionalArguments.first as String;
      if (deleted.contains(id)) {
        return const Left<ApplicationError, NoteView>(
          ApplicationError.notFound(message: 'Not found.'),
        );
      }
      return Right<ApplicationError, NoteView>(
        noteView(id, notebookId: trashed.contains(id) ? 'trash' : 'b_1'),
      );
    });
  });

  NavigationHistoryBloc build() => NavigationHistoryBloc(library, null);

  NavigationHistoryEvent visit(String id) => NavigationHistoryEvent.visited(id);

  NavigationHistoryState stateOf(
    List<String> visits,
    int at, {
    String? pending,
    bool back = false,
    bool forward = false,
  }) {
    return NavigationHistoryState(
      history: NavigationHistory(
        back: visits.sublist(0, at),
        current: visits[at],
        forward: visits.sublist(at + 1).reversed.toList(),
      ),
      pendingNoteId: pending,
      canGoBack: back,
      canGoForward: forward,
    );
  }

  group('NavigationHistoryBloc', () {
    test('starts with no history and nowhere to go', () {
      final state = build().state;
      expect(state.history.current, isNull);
      expect(state.pendingNoteId, isNull);
      expect(state.canGoBack, isFalse);
      expect(state.canGoForward, isFalse);
    });

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'a first visit has nowhere to go',
      build: build,
      act: (bloc) => bloc.add(visit('a')),
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a'], 0),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'the second visit makes going back possible',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b')),
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a'], 0),
        stateOf(<String>['a', 'b'], 1, back: true),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'opening the open note again changes nothing',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('a')),
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a'], 0),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'going back names the note to open, and consumed clears it',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b'))
        ..add(const NavigationHistoryEvent.backRequested())
        ..add(const NavigationHistoryEvent.consumed()),
      skip: 2,
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a', 'b'], 0, pending: 'a', forward: true),
        stateOf(<String>['a', 'b'], 0, forward: true),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'going forward after going back lands on the note left',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b'))
        ..add(const NavigationHistoryEvent.backRequested())
        ..add(const NavigationHistoryEvent.consumed())
        ..add(const NavigationHistoryEvent.forwardRequested()),
      skip: 4,
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a', 'b'], 1, pending: 'b', back: true),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'going back with nowhere to go does nothing',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(const NavigationHistoryEvent.backRequested()),
      expect: () => <NavigationHistoryState>[
        stateOf(<String>['a'], 0),
      ],
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'opening a note after going back drops the visits ahead',
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b'))
        ..add(visit('c'))
        ..add(const NavigationHistoryEvent.backRequested())
        ..add(visit('d')),
      verify: (bloc) {
        expect(bloc.state.history.visits.$1, <String>['a', 'b', 'd']);
        expect(bloc.state.canGoForward, isFalse);
        expect(bloc.state.canGoBack, isTrue);
      },
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'going back skips a note in the trash and one that is deleted',
      setUp: () {
        trashed.add('b');
        deleted.add('c');
      },
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b'))
        ..add(visit('c'))
        ..add(visit('d'))
        ..add(const NavigationHistoryEvent.backRequested()),
      verify: (bloc) {
        expect(bloc.state.pendingNoteId, 'a');
        expect(bloc.state.history.current, 'a');
        expect(bloc.state.history.forward, <String>['d']);
        expect(bloc.state.canGoBack, isFalse);
      },
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'with every note behind gone, going back goes nowhere',
      setUp: () => trashed.addAll(<String>['a', 'b']),
      build: build,
      act: (bloc) => bloc
        ..add(visit('a'))
        ..add(visit('b'))
        ..add(visit('c'))
        ..add(const NavigationHistoryEvent.backRequested()),
      verify: (bloc) {
        expect(bloc.state.pendingNoteId, isNull);
        expect(bloc.state.history.current, 'c');
        expect(bloc.state.canGoBack, isFalse);
      },
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'a locked note is a note that is there',
      setUp: () {
        when(() => library.note('locked')).thenAnswer(
          (_) async => const Right<ApplicationError, NoteView>(
            NoteView(
              id: 'locked',
              rev: '1',
              title: 'Locked',
              lock: LockState.closed(),
              notebookId: 'b_1',
              tags: <String>[],
              status: 'none',
              pinned: false,
              createdMs: 0,
              updatedMs: 0,
            ),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc
        ..add(visit('locked'))
        ..add(visit('b'))
        ..add(const NavigationHistoryEvent.backRequested()),
      verify: (bloc) => expect(bloc.state.pendingNoteId, 'locked'),
    );

    blocTest<NavigationHistoryBloc, NavigationHistoryState>(
      'asks the library only as far as the nearest note that is there',
      build: build,
      act: (bloc) async {
        bloc
          ..add(visit('a'))
          ..add(visit('b'))
          ..add(visit('c'));
        await Future<void>.delayed(const Duration(milliseconds: 20));
        clearInteractions(library);
        bloc.add(visit('d'));
      },
      wait: const Duration(milliseconds: 20),
      verify: (_) {
        // "a" and "b" lie behind "c", which is there: they are not asked
        // for.
        verify(() => library.note('c')).called(1);
        verifyNever(() => library.note('b'));
        verifyNever(() => library.note('a'));
      },
    );
  });
}
