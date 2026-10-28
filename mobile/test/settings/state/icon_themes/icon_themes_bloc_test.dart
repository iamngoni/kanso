// Kanso — icon_themes_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/settings/state/icon_themes/icon_themes_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/seeded_appearance_bloc.dart';

const List<IconThemeRow> _themes = <IconThemeRow>[
  IconThemeRow(
    id: 'lucide',
    name: 'Lucide',
    author: 'Lucide Contributors',
    license: 'ISC',
    preview: <ThemedIcon>[ThemedIcon(name: 'notebook')],
  ),
  IconThemeRow(
    id: 'phosphor',
    name: 'Phosphor',
    author: 'Phosphor Icons',
    license: 'MIT',
    preview: <ThemedIcon>[ThemedIcon(name: 'notebook', svg: '<svg/>')],
  ),
];

void main() {
  late MockRendererRepository renderer;

  setUp(() {
    renderer = MockRendererRepository();
    when(renderer.iconThemes).thenAnswer(
      (_) async => const Right<ApplicationError, List<IconThemeRow>>(_themes),
    );
  });

  IconThemesBloc build() => IconThemesBloc(renderer);

  group('IconThemesBloc', () {
    test('starts loading, with no themes', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.themes, isEmpty);
    });

    blocTest<IconThemesBloc, IconThemesState>(
      'started offers the built-in icon themes in the order the core gives',
      build: build,
      act: (bloc) => bloc.add(const IconThemesEvent.started()),
      expect: () => <IconThemesState>[
        const IconThemesState(),
        const IconThemesState(status: LoadStatus.ready, themes: _themes),
      ],
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'started with no icon themes is ready and empty',
      setUp: () => when(renderer.iconThemes).thenAnswer(
        (_) async => const Right<ApplicationError, List<IconThemeRow>>(
          <IconThemeRow>[],
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const IconThemesEvent.started()),
      expect: () => <IconThemesState>[
        const IconThemesState(),
        const IconThemesState(status: LoadStatus.ready),
      ],
    );

    blocTest<IconThemesBloc, IconThemesState>(
      'started fails when they cannot be read, and reads them again when '
      'asked to try again',
      setUp: () {
        var calls = 0;
        when(renderer.iconThemes).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, List<IconThemeRow>>(
                  ApplicationError.internal(message: 'No icon themes.'),
                )
              : const Right<ApplicationError, List<IconThemeRow>>(_themes);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const IconThemesEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const IconThemesEvent.started());
      },
      expect: () => <IconThemesState>[
        const IconThemesState(),
        const IconThemesState(status: LoadStatus.failed),
        const IconThemesState(),
        const IconThemesState(status: LoadStatus.ready, themes: _themes),
      ],
    );
  });
}
