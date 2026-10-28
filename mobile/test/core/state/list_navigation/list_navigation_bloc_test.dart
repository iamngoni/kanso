// Kanso — list_navigation_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/list_navigation/list_navigation_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

void main() {
  late MockLibraryRepository library;

  setUpAll(() {
    registerFallbackValue(const NoteChange.trash());
  });

  setUp(() {
    library = MockLibraryRepository();
    when(() => library.change(any(), any())).thenAnswer(
      (_) async => const Right<ApplicationError, Changed>(
        Changed(count: 1, message: ''),
      ),
    );
  });

  ListNavigationBloc build() => ListNavigationBloc(library);

  group('ListNavigationBloc', () {
    test('starts with no notes', () {
      expect(build().state.noteIds, isEmpty);
    });

    blocTest<ListNavigationBloc, ListNavigationState>(
      'holds the notes of the list shown, and forgets them',
      build: build,
      act: (bloc) => bloc
        ..add(const ListNavigationEvent.listShown(<String>['a', 'b']))
        ..add(const ListNavigationEvent.listShown(<String>[])),
      expect: () => <ListNavigationState>[
        const ListNavigationState(noteIds: <String>['a', 'b']),
        const ListNavigationState(),
      ],
    );

    blocTest<ListNavigationBloc, ListNavigationState>(
      'a pin request toggles the pin of that note alone',
      build: build,
      act: (bloc) => bloc.add(const ListNavigationEvent.pinRequested('a')),
      expect: () => <ListNavigationState>[],
      verify: (_) {
        verify(
          () => library.change(<String>['a'], const NoteChange.togglePin()),
        ).called(1);
      },
    );

    blocTest<ListNavigationBloc, ListNavigationState>(
      'a pin that fails changes nothing shown',
      setUp: () {
        when(() => library.change(any(), any())).thenAnswer(
          (_) async => const Left<ApplicationError, Changed>(
            ApplicationError.storage(message: 'Disk full.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ListNavigationEvent.pinRequested('a')),
      expect: () => <ListNavigationState>[],
    );
  });
}
