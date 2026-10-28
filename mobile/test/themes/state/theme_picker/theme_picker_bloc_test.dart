// Kanso — theme_picker_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/utils/theme_preview_from_builtin.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_tokens.dart';

BuiltinTheme _theme(String id, {bool dark = false}) {
  return BuiltinTheme(
    id: id,
    name: id.toUpperCase(),
    author: 'Someone',
    license: 'MIT',
    dark: dark,
    tokens: tokensFor(fakePalette(id, dark: dark)),
  );
}

void main() {
  late MockRendererRepository renderer;
  late MockGalleryRepository gallery;
  final themes = <BuiltinTheme>[
    _theme('kanso-light'),
    _theme('nord', dark: true),
  ];

  setUp(() {
    renderer = MockRendererRepository();
    gallery = MockGalleryRepository();
    when(gallery.installedThemes).thenAnswer(
      (_) async => const Right<ApplicationError, List<BuiltinTheme>>(
        <BuiltinTheme>[],
      ),
    );
    when(renderer.builtinThemes).thenAnswer(
      (_) async => Right<ApplicationError, List<BuiltinTheme>>(themes),
    );
  });

  ThemePickerBloc build() => ThemePickerBloc(renderer, gallery);

  group('ThemePickerBloc', () {
    test('starts loading, with no themes', () {
      final state = build().state;
      expect(state.status, LoadStatus.loading);
      expect(state.themes, isEmpty);
    });

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started offers the built-in themes in order, previewed from their '
      'tokens',
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.started()),
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: themes.map(themePreviewFromBuiltin).toList(),
        ),
      ],
      verify: (bloc) {
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
        ]);
        expect(bloc.state.themes.last.dark, isTrue);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started adds the themes installed from the gallery after the built-in '
      'ones, previewed from their tokens, and not marked as built in',
      setUp: () => when(gallery.installedThemes).thenAnswer(
        (_) async => Right<ApplicationError, List<BuiltinTheme>>(
          <BuiltinTheme>[_theme('dusk', dark: true)],
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.started()),
      verify: (bloc) {
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
          'dusk',
        ]);
        expect(bloc.state.themes.map((theme) => theme.builtIn), <bool>[
          true,
          true,
          false,
        ]);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'installed themes that cannot be read leave the built-in ones',
      setUp: () => when(gallery.installedThemes).thenAnswer(
        (_) async => const Left<ApplicationError, List<BuiltinTheme>>(
          ApplicationError.internal(message: 'No gallery.'),
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.started()),
      verify: (bloc) {
        expect(bloc.state.status, LoadStatus.ready);
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
        ]);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'installedChanged reads the installed themes again and keeps the '
      'built-in ones as they were',
      build: build,
      act: (bloc) async {
        bloc.add(const ThemePickerEvent.started());
        await Future<void>.delayed(Duration.zero);
        when(gallery.installedThemes).thenAnswer(
          (_) async => Right<ApplicationError, List<BuiltinTheme>>(
            <BuiltinTheme>[_theme('dusk', dark: true), _theme('moss')],
          ),
        );
        bloc.add(const ThemePickerEvent.installedChanged());
        await Future<void>.delayed(Duration.zero);
        when(gallery.installedThemes).thenAnswer(
          (_) async => Right<ApplicationError, List<BuiltinTheme>>(
            <BuiltinTheme>[_theme('moss')],
          ),
        );
        bloc.add(const ThemePickerEvent.installedChanged());
      },
      verify: (bloc) {
        expect(bloc.state.themes.map((theme) => theme.id), <String>[
          'kanso-light',
          'nord',
          'moss',
        ]);
        verify(() => renderer.builtinThemes()).called(1);
      },
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'installedChanged before the picker has read anything does nothing',
      build: build,
      act: (bloc) => bloc.add(const ThemePickerEvent.installedChanged()),
      expect: () => <ThemePickerState>[],
    );

    blocTest<ThemePickerBloc, ThemePickerState>(
      'started fails when the themes cannot be read, and reads them again '
      'when asked to try again',
      setUp: () {
        var calls = 0;
        when(renderer.builtinThemes).thenAnswer((_) async {
          calls++;
          return calls == 1
              ? const Left<ApplicationError, List<BuiltinTheme>>(
                  ApplicationError.internal(message: 'No themes.'),
                )
              : Right<ApplicationError, List<BuiltinTheme>>(themes);
        });
      },
      build: build,
      act: (bloc) async {
        bloc.add(const ThemePickerEvent.started());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const ThemePickerEvent.started());
      },
      expect: () => <ThemePickerState>[
        const ThemePickerState(),
        const ThemePickerState(status: LoadStatus.failed),
        const ThemePickerState(),
        ThemePickerState(
          status: LoadStatus.ready,
          themes: themes.map(themePreviewFromBuiltin).toList(),
        ),
      ],
    );
  });
}
