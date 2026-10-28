// Kanso — theme_gallery_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/models/data/gallery_filter.dart';
import 'package:kanso/themes/models/data/gallery_problem.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/gallery_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/settle.dart';

Either<ApplicationError, List<GalleryItem>> _items(List<GalleryItem> items) =>
    Right<ApplicationError, List<GalleryItem>>(items);

Either<ApplicationError, List<GalleryItem>> _bad(ApplicationError error) =>
    Left<ApplicationError, List<GalleryItem>>(error);

void main() {
  late MockGalleryRepository gallery;
  final dusk = galleryItem(
    'dusk',
    name: 'Dusk',
    official: true,
    dark: true,
    updatedMs: 1,
  );
  final paper = galleryItem('paper', name: 'Paper', updatedMs: 5);
  final installedPaper = galleryItem(
    'paper',
    name: 'Paper',
    updatedMs: 5,
    installed: true,
  );

  setUp(() {
    gallery = MockGalleryRepository();
    when(() => gallery.index()).thenAnswer(
      (_) async => _items(<GalleryItem>[dusk, paper]),
    );
    when(() => gallery.index(refresh: true)).thenAnswer(
      (_) async => _items(<GalleryItem>[dusk, paper]),
    );
  });

  ThemeGalleryBloc build() => ThemeGalleryBloc(gallery);

  group('ThemeGalleryBloc loading', () {
    test('starts loading, with nothing shown', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.items, isEmpty);
      expect(state.shown, isEmpty);
      expect(state.filter, GalleryFilter.popular);
    });

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'shows the last index at once, then fetches the current one behind it',
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.started()),
      expect: () => <ThemeGalleryState>[
        const ThemeGalleryState(),
        ThemeGalleryState(
          status: LoadStatus.ready,
          items: <GalleryItem>[dusk, paper],
          refreshing: true,
        ),
        ThemeGalleryState(
          status: LoadStatus.ready,
          items: <GalleryItem>[dusk, paper],
        ),
      ],
      verify: (_) => verifyInOrder(<void Function()>[
        () => gallery.index(),
        () => gallery.index(refresh: true),
      ]),
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'new items in the current index replace the old ones',
      setUp: () => when(() => gallery.index(refresh: true)).thenAnswer(
        (_) async => _items(<GalleryItem>[dusk, paper, galleryItem('moss')]),
      ),
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.started()),
      verify: (bloc) => expect(bloc.state.items.map((i) => i.id), <String>[
        'dusk',
        'paper',
        'moss',
      ]),
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'fails when no index can be read, and does not ask for a fresh one',
      setUp: () => when(() => gallery.index()).thenAnswer(
        (_) async => _bad(const ApplicationError.network(message: 'Off.')),
      ),
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.started()),
      expect: () => <ThemeGalleryState>[
        const ThemeGalleryState(),
        const ThemeGalleryState(status: LoadStatus.failed),
      ],
      verify: (_) => verifyNever(() => gallery.index(refresh: true)),
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'when the fresh index cannot be fetched the last one stays, with a note',
      setUp: () => when(() => gallery.index(refresh: true)).thenAnswer(
        (_) async => _bad(const ApplicationError.network(message: 'Off.')),
      ),
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.started()),
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.items, <GalleryItem>[dusk, paper]);
        expect(bloc.state.refreshFailed, isTrue);
        expect(bloc.state.refreshing, isFalse);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'fetching again after that clears the note',
      setUp: () {
        var fetches = 0;
        when(() => gallery.index(refresh: true)).thenAnswer((_) async {
          fetches++;
          return fetches == 1
              ? _bad(const ApplicationError.network(message: 'Off.'))
              : _items(<GalleryItem>[dusk]);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.refreshed());
      },
      verify: (bloc) {
        expect(bloc.state.refreshFailed, isFalse);
        expect(bloc.state.items, <GalleryItem>[dusk]);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'asking to fetch again with nothing shown yet does nothing',
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.refreshed()),
      expect: () => <ThemeGalleryState>[],
      verify: (_) => verifyNever(() => gallery.index(refresh: true)),
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'trying again after a failure reads it again and keeps the search',
      setUp: () {
        var reads = 0;
        when(() => gallery.index()).thenAnswer((_) async {
          reads++;
          return reads == 1
              ? _bad(const ApplicationError.network(message: 'Off.'))
              : _items(<GalleryItem>[dusk, paper]);
        });
      },
      build: build,
      act: (bloc) async {
        bloc
          ..add(const ThemeGalleryEvent.searched('pap'))
          ..add(const ThemeGalleryEvent.started());
        await settle();
        expect(bloc.state.status, LoadStatus.failed);
        bloc.add(const ThemeGalleryEvent.started());
      },
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.query, 'pap');
        expect(bloc.state.shown.map((i) => i.id), <String>['paper']);
      },
    );
  });

  group('ThemeGalleryBloc searching and ordering', () {
    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'the search narrows what is shown, not what is read',
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.searched('dusk'));
      },
      verify: (bloc) {
        expect(bloc.state.shown.map((i) => i.id), <String>['dusk']);
        expect(bloc.state.items.length, 2);
        verify(() => gallery.index()).called(1);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'a search with no match leaves nothing shown',
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.searched('zzz'));
      },
      verify: (bloc) {
        expect(bloc.state.shown, isEmpty);
        expect(bloc.state.items, isNotEmpty);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'the filters order the items',
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        expect(bloc.state.shown.map((i) => i.id), <String>['dusk', 'paper']);
        bloc.add(const ThemeGalleryEvent.filterChosen(GalleryFilter.recent));
      },
      verify: (bloc) {
        expect(bloc.state.filter, GalleryFilter.recent);
        // Paper was updated after Dusk.
        expect(bloc.state.shown.map((i) => i.id), <String>['paper', 'dusk']);
      },
    );
  });

  group('ThemeGalleryBloc installing', () {
    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'installs a theme: working while it goes, then installed, and the '
      'picker is told which theme changed',
      setUp: () => when(() => gallery.install('paper')).thenAnswer(
        (_) async => _items(<GalleryItem>[dusk, installedPaper]),
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.installed('paper'));
      },
      verify: (bloc) {
        expect(bloc.state.working, isEmpty);
        expect(bloc.state.problem, isNull);
        expect(bloc.state.items.last.installed, isTrue);
        expect(bloc.state.installedRevision, 1);
        expect(bloc.state.changed, <String>{'paper'});
      },
    );

    test(
      'a theme is working from when Install is pressed until it is done',
      () async {
        final gate = Completer<void>();
        when(() => gallery.install('paper')).thenAnswer((_) async {
          await gate.future;
          return _items(<GalleryItem>[dusk, installedPaper]);
        });
        final bloc = build()..add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.installed('paper'));
        await settle();
        expect(bloc.state.working, <String>{'paper'});
        gate.complete();
        await settle();
        expect(bloc.state.working, isEmpty);
        await bloc.close();
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'a theme that cannot be installed stays as it was, with the reason',
      setUp: () => when(() => gallery.install('paper')).thenAnswer(
        (_) async => _bad(
          const ApplicationError.invalid(message: 'Paper needs a newer Kanso.'),
        ),
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.installed('paper'));
      },
      verify: (bloc) {
        expect(bloc.state.items, <GalleryItem>[dusk, paper]);
        expect(
          bloc.state.problem,
          const GalleryProblem.words('Paper needs a newer Kanso.'),
        );
        expect(bloc.state.working, isEmpty);
        expect(bloc.state.installedRevision, 0);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'an install with no network says so in the app’s words',
      setUp: () => when(() => gallery.install('paper')).thenAnswer(
        (_) async => _bad(const ApplicationError.network(message: 'dns')),
      ),
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.installed('paper'));
      },
      verify: (bloc) =>
          expect(bloc.state.problem, const GalleryProblem.offline()),
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'removes a theme',
      setUp: () {
        when(() => gallery.index()).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, installedPaper]),
        );
        when(() => gallery.index(refresh: true)).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, installedPaper]),
        );
        when(() => gallery.uninstall('paper')).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, paper]),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc.add(const ThemeGalleryEvent.uninstalled('paper'));
      },
      verify: (bloc) {
        expect(bloc.state.items.last.installed, isFalse);
        expect(bloc.state.changed, <String>{'paper'});
        expect(bloc.state.installedRevision, 1);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'a theme the catalog withdrew shows as changed when the fresh index '
      'comes',
      setUp: () {
        when(() => gallery.index()).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, installedPaper]),
        );
        when(() => gallery.index(refresh: true)).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, paper]),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const ThemeGalleryEvent.started()),
      verify: (bloc) {
        expect(bloc.state.changed, <String>{'paper'});
        expect(bloc.state.installedRevision, 1);
      },
    );

    blocTest<ThemeGalleryBloc, ThemeGalleryState>(
      'steps go one after another, in the order they came',
      setUp: () {
        when(() => gallery.install(any())).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, installedPaper]),
        );
        when(() => gallery.uninstall(any())).thenAnswer(
          (_) async => _items(<GalleryItem>[dusk, paper]),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ThemeGalleryEvent.started());
        await settle();
        bloc
          ..add(const ThemeGalleryEvent.installed('paper'))
          ..add(const ThemeGalleryEvent.uninstalled('paper'));
      },
      verify: (bloc) {
        verifyInOrder(<void Function()>[
          () => gallery.install('paper'),
          () => gallery.uninstall('paper'),
        ]);
        expect(bloc.state.items.last.installed, isFalse);
      },
    );
  });
}
