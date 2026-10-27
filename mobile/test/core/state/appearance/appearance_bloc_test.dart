// Kanso — appearance_bloc_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart' show Color, ThemeMode;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' show DiagramRenderer;
import 'package:kanso/core/bridge/api/types.dart' show SortKey;
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_syntax_colors.dart';
import 'package:kanso/core/models/data/appearance_status.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_icon_set.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_tokens.dart';

final KansoPalette _nord = fakePalette('nord', dark: true);
final KansoPalette _github = fakePalette('github-light');

const IconThemeRow _phosphorRow = IconThemeRow(
  id: 'phosphor',
  name: 'Phosphor',
  author: 'Phosphor Icons',
  license: 'MIT',
  preview: <ThemedIcon>[],
);

/// Phosphor as the bloc keeps it: only the icons it draws.
const KansoIconSet _phosphor = KansoIconSet(
  id: 'phosphor',
  name: 'Phosphor',
  svgs: <String, String>{'search': '<svg id="search"/>'},
);

void main() {
  late MockPreferencesRepository preferences;
  late MockRendererRepository renderer;
  late MockGalleryRepository gallery;
  late Preferences saved;

  setUpAll(() {
    registerFallbackValue(const Preferences());
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    preferences = MockPreferencesRepository();
    renderer = MockRendererRepository();
    gallery = MockGalleryRepository();
    when(
      () => gallery.themeChanges(),
    ).thenAnswer((_) => const Stream<String>.empty());
    // No theme is installed from the gallery unless a test says so.
    when(() => gallery.installedThemeTokens(any())).thenAnswer(
      (_) async => const Left<ApplicationError, ThemeTokens>(
        ApplicationError.notFound(message: 'Not installed.'),
      ),
    );
    saved = const Preferences();
    when(
      () => preferences.load(),
    ).thenAnswer((_) async => Right<ApplicationError, Preferences>(saved));
    when(() => preferences.update(any())).thenAnswer((call) async {
      final change =
          call.positionalArguments.first as Preferences Function(Preferences);
      saved = change(saved);
      return Right<ApplicationError, Preferences>(saved);
    });
    when(() => renderer.builtinThemeTokens('nord')).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(tokensFor(_nord)),
    );
    when(() => renderer.builtinThemeTokens('github-light')).thenAnswer(
      (_) async => Right<ApplicationError, ThemeTokens>(tokensFor(_github)),
    );
    when(() => renderer.iconThemes()).thenAnswer(
      (_) async => const Right<ApplicationError, List<IconThemeRow>>(
        <IconThemeRow>[_phosphorRow],
      ),
    );
    when(() => renderer.iconThemeIcons('phosphor')).thenAnswer(
      (_) async => const Right<ApplicationError, List<ThemedIcon>>(
        <ThemedIcon>[
          ThemedIcon(name: 'search', svg: '<svg id="search"/>'),
          ThemedIcon(name: 'pin'),
        ],
      ),
    );
    when(() => renderer.iconThemeIcons('gone')).thenAnswer(
      (_) async => const Left<ApplicationError, List<ThemedIcon>>(
        ApplicationError.notFound(message: 'No such icon theme.'),
      ),
    );
    when(() => renderer.builtinThemeTokens('gone')).thenAnswer(
      (_) async => const Left<ApplicationError, ThemeTokens>(
        ApplicationError.notFound(message: 'No such theme.'),
      ),
    );
  });

  AppearanceBloc build() => AppearanceBloc(preferences, renderer, gallery);

  group('AppearanceBloc', () {
    test('starts loading, in the Kanso themes, following the system', () {
      final state = build().state;
      expect(state.status, AppearanceStatus.loading);
      expect(state.light, KansoPalette.kansoLight);
      expect(state.dark, KansoPalette.kansoDark);
      expect(state.selection, const ThemeSelection());
      expect(state.themeMode, ThemeMode.system);
      expect(state.fonts, const KansoFonts());
      expect(state.textSize, KansoTextSize.standard);
    });

    blocTest<AppearanceBloc, AppearanceState>(
      'started shows the defaults as ready when nothing was saved',
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        const AppearanceState(status: AppearanceStatus.ready),
      ],
      verify: (_) {
        expect(themes.fits, isEmpty);
        expect(themes.renderersSet, <DiagramRenderer>[DiagramRenderer.pele]);
        verifyNever(() => preferences.update(any()));
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started shows what was saved, reading the themes it names',
      setUp: () {
        saved = const Preferences(
          themeSelection: ThemeSelection(
            followSystem: false,
            ui: 'nord',
            light: 'github-light',
            dark: 'nord',
          ),
          fonts: KansoFonts(editorFont: KansoFontFamily.literata),
          textSize: KansoTextSize.large,
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: saved.themeSelection,
          light: _nord,
          dark: _nord,
          fonts: saved.fonts,
          textSize: KansoTextSize.large,
          status: AppearanceStatus.ready,
        ),
      ],
      verify: (bloc) {
        expect(bloc.state.themeMode, ThemeMode.dark);
        verify(() => renderer.builtinThemeTokens('nord')).called(1);
        // The slot not shown is read too, to know it is still there; it is
        // read once and kept.
        verify(() => renderer.builtinThemeTokens('github-light')).called(1);
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started following the system reads both slots',
      setUp: () {
        saved = const Preferences(
          themeSelection: ThemeSelection(light: 'github-light', dark: 'nord'),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: saved.themeSelection,
          light: _github,
          dark: _nord,
          status: AppearanceStatus.ready,
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started with unreadable preferences shows the defaults and says failed',
      setUp: () {
        when(() => preferences.load()).thenAnswer(
          (_) async => const Left<ApplicationError, Preferences>(
            ApplicationError.storage(message: 'Disk error.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        const AppearanceState(status: AppearanceStatus.failed),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'picking a dark theme while following takes the dark slot and saves',
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.themePicked('nord', dark: true)),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(ui: 'nord', dark: 'nord'),
          dark: _nord,
        ),
      ],
      verify: (_) {
        expect(
          saved.themeSelection,
          const ThemeSelection(ui: 'nord', dark: 'nord'),
        );
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'picking a light theme while following takes the light slot',
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.themePicked('github-light', dark: false),
      ),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(
            ui: 'github-light',
            light: 'github-light',
          ),
          light: _github,
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'picking a theme while not following makes both themes that one',
      seed: () => const AppearanceState(
        selection: ThemeSelection(followSystem: false),
      ),
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.themePicked('nord', dark: true)),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(followSystem: false, ui: 'nord'),
          light: _nord,
          dark: _nord,
        ),
      ],
      verify: (bloc) => expect(bloc.state.themeMode, ThemeMode.dark),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'picking a theme that cannot be read changes and saves nothing',
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.themePicked('gone', dark: true)),
      expect: () => <AppearanceState>[],
      verify: (_) => verifyNever(() => preferences.update(any())),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'turning following off shows the fixed theme in both slots',
      seed: () => const AppearanceState(
        selection: ThemeSelection(ui: 'nord', dark: 'nord'),
      ),
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.followSystemChanged(follow: false)),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(
            followSystem: false,
            ui: 'nord',
            dark: 'nord',
          ),
          light: _nord,
          dark: _nord,
        ),
      ],
      verify: (_) => expect(saved.themeSelection.followSystem, isFalse),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'turning following on again brings back the two slots',
      seed: () => AppearanceState(
        selection: const ThemeSelection(
          followSystem: false,
          ui: 'nord',
          light: 'github-light',
        ),
        light: _nord,
        dark: _nord,
      ),
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.followSystemChanged(follow: true)),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(ui: 'nord', light: 'github-light'),
          light: _github,
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'the same choice twice emits once but is read from the core once',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.themePicked('nord', dark: true))
        ..add(const AppearanceEvent.themePicked('nord', dark: true)),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: const ThemeSelection(ui: 'nord', dark: 'nord'),
          dark: _nord,
        ),
      ],
      verify: (_) => expect(themes.fits, <(String, String, String)>[
        ('nord', 'nord', 'nord'),
      ]),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'choices of different kinds are applied in the order they came',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.followSystemChanged(follow: false))
        ..add(const AppearanceEvent.themePicked('nord', dark: true))
        ..add(const AppearanceEvent.followSystemChanged(follow: true)),
      verify: (bloc) {
        expect(
          bloc.state.selection,
          const ThemeSelection(ui: 'nord'),
        );
        expect(saved.themeSelection, bloc.state.selection);
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      "started draws Kanso's own icons without asking the core for any",
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      verify: (bloc) {
        expect(bloc.state.iconSet, KansoIconSet.lucide);
        verifyNever(() => renderer.iconThemes());
        verifyNever(() => renderer.iconThemeIcons(any()));
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'started shows the icon theme that was saved, with the icons it draws',
      setUp: () => saved = const Preferences(iconTheme: 'phosphor'),
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        const AppearanceState(
          iconSet: _phosphor,
          status: AppearanceStatus.ready,
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      "started with an icon theme that cannot be read shows Kanso's own and "
      'says it failed',
      setUp: () => saved = const Preferences(iconTheme: 'gone'),
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        const AppearanceState(status: AppearanceStatus.failed),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'iconThemePicked shows the icon theme at once and saves its ID',
      build: build,
      act: (bloc) =>
          bloc.add(const AppearanceEvent.iconThemePicked('phosphor')),
      expect: () => <AppearanceState>[
        const AppearanceState(iconSet: _phosphor),
      ],
      verify: (_) => expect(saved.iconTheme, 'phosphor'),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      "iconThemePicked back to Lucide draws Kanso's own icons and saves it",
      setUp: () => saved = const Preferences(iconTheme: 'phosphor'),
      build: build,
      seed: () => const AppearanceState(iconSet: _phosphor),
      act: (bloc) => bloc.add(const AppearanceEvent.iconThemePicked('lucide')),
      expect: () => <AppearanceState>[const AppearanceState()],
      verify: (_) => expect(saved.iconTheme, 'lucide'),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'iconThemePicked reads an icon theme once and keeps it',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.iconThemePicked('phosphor'))
        ..add(const AppearanceEvent.iconThemePicked('lucide'))
        ..add(const AppearanceEvent.iconThemePicked('phosphor')),
      verify: (bloc) {
        expect(bloc.state.iconSet, _phosphor);
        verify(() => renderer.iconThemeIcons('phosphor')).called(1);
      },
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'iconThemePicked for a theme that is not there changes nothing',
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.iconThemePicked('gone')),
      expect: () => <AppearanceState>[],
      verify: (_) => verifyNever(() => preferences.update(any())),
    );

    test('a palette is the mapping of its tokens', () {
      expect(paletteFromTokens('nord', tokensFor(_nord)), _nord);
    });

    test('a choice that cannot be saved is still shown', () async {
      when(() => preferences.update(any())).thenAnswer(
        (_) async => const Left<ApplicationError, Preferences>(
          ApplicationError.storage(message: 'Disk full.'),
        ),
      );
      final bloc = build()
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.large));
      await bloc.stream.first;
      expect(bloc.state.textSize, KansoTextSize.large);
      await bloc.close();
    });

    blocTest<AppearanceBloc, AppearanceState>(
      'editorFontChanged changes only the editor font and saves it',
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.editorFontChanged(KansoFontFamily.literata),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(editorFont: KansoFontFamily.literata),
        ),
      ],
      verify: (_) => expect(
        saved.fonts,
        const KansoFonts(editorFont: KansoFontFamily.literata),
      ),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      "editorFontChanged accepts the device's font",
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.editorFontChanged(KansoFontFamily.system),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(editorFont: KansoFontFamily.system),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'previewFontChanged changes only the preview font',
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.previewFontChanged(KansoFontFamily.sourceSerif4),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.sourceSerif4),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'codeFontChanged changes only the code font',
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.codeFontChanged(KansoFontFamily.ibmPlexMono),
      ),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(codeFont: KansoFontFamily.ibmPlexMono),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'codeFontChanged ignores a family that is not monospaced',
      build: build,
      act: (bloc) => bloc.add(
        const AppearanceEvent.codeFontChanged(KansoFontFamily.inter),
      ),
      expect: () => <AppearanceState>[],
      verify: (_) => verifyNever(() => preferences.update(any())),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'textSizeChanged emits each size chosen and saves the last',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.large))
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.small)),
      expect: () => <AppearanceState>[
        const AppearanceState(textSize: KansoTextSize.large),
        const AppearanceState(textSize: KansoTextSize.small),
      ],
      verify: (_) => expect(saved.textSize, KansoTextSize.small),
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'choosing the same font twice emits once',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter))
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter)),
      expect: () => <AppearanceState>[
        const AppearanceState(
          fonts: KansoFonts(previewFont: KansoFontFamily.inter),
        ),
      ],
    );

    blocTest<AppearanceBloc, AppearanceState>(
      'font, size and theme choices keep each other and all are saved',
      build: build,
      act: (bloc) => bloc
        ..add(const AppearanceEvent.themePicked('nord', dark: true))
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.larger))
        ..add(const AppearanceEvent.editorFontChanged(KansoFontFamily.inter))
        ..add(const AppearanceEvent.previewFontChanged(KansoFontFamily.inter)),
      verify: (bloc) {
        expect(
          bloc.state,
          AppearanceState(
            selection: const ThemeSelection(ui: 'nord', dark: 'nord'),
            dark: _nord,
            textSize: KansoTextSize.larger,
            fonts: const KansoFonts(
              editorFont: KansoFontFamily.inter,
              previewFont: KansoFontFamily.inter,
            ),
          ),
        );
        expect(
          saved,
          Preferences(
            themeSelection: bloc.state.selection,
            textSize: KansoTextSize.larger,
            fonts: bloc.state.fonts,
          ),
        );
      },
    );

    group('with themes installed from the gallery', () {
      final dusk = fakePalette('dusk', dark: true);
      final duskUpdated = dusk.copyWith(name: 'Dusk, updated');

      setUp(() {
        when(() => renderer.builtinThemeTokens('dusk')).thenAnswer(
          (_) async => const Left<ApplicationError, ThemeTokens>(
            ApplicationError.notFound(message: 'Not built in.'),
          ),
        );
        when(() => gallery.installedThemeTokens('dusk')).thenAnswer(
          (_) async => Right<ApplicationError, ThemeTokens>(tokensFor(dusk)),
        );
      });

      blocTest<AppearanceBloc, AppearanceState>(
        'picking an installed theme reads it from the gallery when it is not '
        'built in',
        build: build,
        act: (bloc) =>
            bloc.add(const AppearanceEvent.themePicked('dusk', dark: true)),
        expect: () => <AppearanceState>[
          AppearanceState(
            selection: const ThemeSelection(ui: 'dusk', dark: 'dusk'),
            dark: paletteFromTokens('dusk', tokensFor(dusk)),
          ),
        ],
        verify: (_) {
          verify(() => gallery.installedThemeTokens('dusk')).called(1);
          expect(saved.themeSelection.dark, 'dusk');
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'a built-in theme is never looked for in the gallery',
        build: build,
        act: (bloc) =>
            bloc.add(const AppearanceEvent.themePicked('nord', dark: true)),
        verify: (_) => verifyNever(() => gallery.installedThemeTokens(any())),
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'a theme in use that was updated is drawn in its new colours',
        seed: () => AppearanceState(
          selection: const ThemeSelection(ui: 'dusk', dark: 'dusk'),
          dark: paletteFromTokens('dusk', tokensFor(dusk)),
        ),
        setUp: () {
          when(() => gallery.installedThemeTokens('dusk')).thenAnswer(
            (_) async =>
                Right<ApplicationError, ThemeTokens>(tokensFor(duskUpdated)),
          );
        },
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.themeChanged('dusk')),
        expect: () => <AppearanceState>[
          AppearanceState(
            selection: const ThemeSelection(ui: 'dusk', dark: 'dusk'),
            dark: paletteFromTokens('dusk', tokensFor(duskUpdated)),
          ),
        ],
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'a theme in use that was removed goes back to Kanso’s own, in every '
        'slot that held it, and the choice is saved',
        seed: () => AppearanceState(
          selection: const ThemeSelection(
            followSystem: false,
            ui: 'dusk',
            light: 'dusk',
            dark: 'dusk',
          ),
          light: paletteFromTokens('dusk', tokensFor(dusk)),
          dark: paletteFromTokens('dusk', tokensFor(dusk)),
        ),
        setUp: () {
          when(() => gallery.installedThemeTokens('dusk')).thenAnswer(
            (_) async => const Left<ApplicationError, ThemeTokens>(
              ApplicationError.notFound(message: 'Removed.'),
            ),
          );
        },
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.themeChanged('dusk')),
        expect: () => <AppearanceState>[
          const AppearanceState(
            selection: ThemeSelection(followSystem: false),
            light: KansoPalette.kansoDark,
          ),
        ],
        verify: (bloc) {
          expect(saved.themeSelection.ui, ThemeSelection.kansoDark);
          expect(saved.themeSelection.light, ThemeSelection.kansoLight);
          expect(saved.themeSelection.dark, ThemeSelection.kansoDark);
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'a removed theme in a slot that is not showing goes back to Kanso’s '
        'own too, so it is not back when the system’s appearance changes',
        // Following the system shows the light and dark slots; the fixed
        // one, for when following stops, holds the removed theme.
        seed: () => const AppearanceState(
          selection: ThemeSelection(ui: 'dusk'),
        ),
        setUp: () {
          when(() => gallery.installedThemeTokens('dusk')).thenAnswer(
            (_) async => const Left<ApplicationError, ThemeTokens>(
              ApplicationError.notFound(message: 'Removed.'),
            ),
          );
        },
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.themeChanged('dusk')),
        verify: (bloc) {
          expect(bloc.state.selection, const ThemeSelection());
          expect(saved.themeSelection, const ThemeSelection());
        },
      );

      test('a theme the gallery reports changed, wherever it was changed, '
          'reaches the app', () async {
        final changes = StreamController<String>.broadcast();
        addTearDown(changes.close);
        when(() => gallery.themeChanges()).thenAnswer((_) => changes.stream);
        when(() => gallery.installedThemeTokens('dusk')).thenAnswer(
          (_) async => const Left<ApplicationError, ThemeTokens>(
            ApplicationError.notFound(message: 'Withdrawn.'),
          ),
        );
        saved = const Preferences(
          themeSelection: ThemeSelection(followSystem: false, ui: 'nord'),
        );
        final bloc = build()..add(const AppearanceEvent.started());
        addTearDown(bloc.close);
        await bloc.stream.firstWhere(
          (state) => state.status == AppearanceStatus.ready,
        );
        bloc.add(const AppearanceEvent.themePicked('nord', dark: true));
        // The catalog withdrew a theme in use, and nobody had the picker
        // open: the gallery alone says so.
        when(() => renderer.builtinThemeTokens('nord')).thenAnswer(
          (_) async => const Left<ApplicationError, ThemeTokens>(
            ApplicationError.notFound(message: 'Gone.'),
          ),
        );
        changes.add('nord');
        await bloc.stream.firstWhere(
          (state) => state.selection.ui == ThemeSelection.kansoDark,
        );
        expect(saved.themeSelection.ui, ThemeSelection.kansoDark);
      });

      test('stops listening to the gallery once it is closed', () async {
        final changes = StreamController<String>.broadcast();
        addTearDown(changes.close);
        when(() => gallery.themeChanges()).thenAnswer((_) => changes.stream);
        final bloc = build();
        expect(changes.hasListener, isTrue);
        await bloc.close();
        expect(changes.hasListener, isFalse);
      });

      blocTest<AppearanceBloc, AppearanceState>(
        'a theme that is not in use changing changes nothing',
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.themeChanged('dusk')),
        expect: () => <AppearanceState>[],
        verify: (_) => verifyNever(() => preferences.update(any())),
      );
    });

    test('a saved theme that cannot be read at start is Kanso’s own from '
        'then on, in every slot that held it, and that is saved', () async {
      saved = const Preferences(
        themeSelection: ThemeSelection(ui: 'gone', light: 'gone'),
      );
      final bloc = build()..add(const AppearanceEvent.started());
      addTearDown(bloc.close);
      final ready = await bloc.stream.firstWhere(
        (state) => state.status != AppearanceStatus.loading,
      );
      expect(ready.status, AppearanceStatus.ready);
      expect(ready.selection, const ThemeSelection());
      expect(ready.light, KansoPalette.kansoLight);
      await Future<void>.delayed(Duration.zero);
      expect(saved.themeSelection, const ThemeSelection());
    });

    test('a start whose themes all read saves nothing', () async {
      saved = const Preferences(
        themeSelection: ThemeSelection(followSystem: false, ui: 'nord'),
      );
      final bloc = build()..add(const AppearanceEvent.started());
      addTearDown(bloc.close);
      await bloc.stream.firstWhere(
        (state) => state.status != AppearanceStatus.loading,
      );
      await Future<void>.delayed(Duration.zero);
      verifyNever(() => preferences.update(any()));
    });

    test('does not touch what it does not own in the preferences', () async {
      saved = const Preferences(
        sort: SortKey.title,
        descending: false,
        backupNoticeSeen: true,
      );
      final bloc = build()
        ..add(const AppearanceEvent.textSizeChanged(KansoTextSize.large));
      await bloc.stream.first;
      await Future<void>.delayed(Duration.zero);
      expect(saved.sort, SortKey.title);
      expect(saved.descending, isFalse);
      expect(saved.backupNoticeSeen, isTrue);
      await bloc.close();
    });

    group('the syntax and preview themes', () {
      blocTest<AppearanceBloc, AppearanceState>(
        'picking a syntax theme fits it with the interface and keeps the '
        'interface colours',
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.syntaxPicked('nord')),
        verify: (bloc) {
          final state = bloc.state;
          expect(state.selection.syntax, 'nord');
          expect(state.selection.preview, isNull);
          expect(state.light.id, 'kanso-light');
          expect(state.light.colors, KansoPalette.kansoLight.colors);
          // The Markdown is coloured by the syntax theme's tokens (the code
          // block's background is the preview theme's).
          expect(_tokensOnly(state.light.syntax), _tokensOnly(_nord.syntax));
          expect(state.light.preview, KansoPalette.kansoLight.preview);
          expect(_tokensOnly(state.dark.syntax), _tokensOnly(_nord.syntax));
          expect(themes.fits, contains(('kanso-light', 'nord', 'kanso-light')));
          expect(themes.fits, contains(('kanso-dark', 'nord', 'kanso-dark')));
          expect(saved.themeSelection.syntax, 'nord');
          expect(state.syntaxThemeId(dark: false), 'nord');
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'picking a preview theme draws the preview in its colours alone',
        build: build,
        act: (bloc) =>
            bloc.add(const AppearanceEvent.previewPicked('github-light')),
        verify: (bloc) {
          final state = bloc.state;
          expect(state.selection.preview, 'github-light');
          expect(state.dark.colors, KansoPalette.kansoDark.colors);
          expect(
            _tokensOnly(state.dark.syntax),
            _tokensOnly(KansoPalette.kansoDark.syntax),
          );
          expect(
            state.dark.syntax.codeBackground,
            Color(0xFF000000 | _github.preview.surface),
          );
          expect(state.dark.preview, _github.preview);
          expect(state.light.preview, _github.preview);
          expect(saved.themeSelection.preview, 'github-light');
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'picking none goes back to the interface theme and saves it',
        seed: () => const AppearanceState(
          selection: ThemeSelection(syntax: 'nord', preview: 'nord'),
        ),
        build: build,
        act: (bloc) => bloc
          ..add(const AppearanceEvent.syntaxPicked(null))
          ..add(const AppearanceEvent.previewPicked(null)),
        verify: (bloc) {
          expect(bloc.state.selection, const ThemeSelection());
          expect(bloc.state.light, KansoPalette.kansoLight);
          expect(bloc.state.dark, KansoPalette.kansoDark);
          expect(saved.themeSelection, const ThemeSelection());
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'a syntax theme that cannot be read changes and saves nothing',
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.syntaxPicked('gone')),
        expect: () => <AppearanceState>[],
        verify: (_) => verifyNever(() => preferences.update(any())),
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'the syntax and preview themes survive picking an interface theme',
        build: build,
        act: (bloc) => bloc
          ..add(const AppearanceEvent.syntaxPicked('nord'))
          ..add(const AppearanceEvent.previewPicked('github-light'))
          ..add(const AppearanceEvent.themePicked('nord', dark: true)),
        verify: (bloc) {
          expect(bloc.state.selection.syntax, 'nord');
          expect(bloc.state.selection.preview, 'github-light');
          expect(bloc.state.dark.id, 'nord');
          expect(bloc.state.dark.preview, _github.preview);
          expect(
            themes.fits,
            contains(('nord', 'nord', 'github-light')),
          );
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'started shows the saved syntax and preview themes',
        setUp: () {
          saved = const Preferences(
            themeSelection: ThemeSelection(
              syntax: 'nord',
              preview: 'github-light',
            ),
          );
        },
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.started()),
        verify: (bloc) {
          expect(bloc.state.status, AppearanceStatus.ready);
          expect(
            _tokensOnly(bloc.state.light.syntax),
            _tokensOnly(_nord.syntax),
          );
          expect(bloc.state.light.preview, _github.preview);
          expect(bloc.state.dark.preview, _github.preview);
        },
      );
    });

    group('the preview text size and the diagrams', () {
      blocTest<AppearanceBloc, AppearanceState>(
        'previewTextSizeChanged changes only the preview text size and saves '
        'it',
        build: build,
        act: (bloc) => bloc.add(
          const AppearanceEvent.previewTextSizeChanged(KansoTextSize.larger),
        ),
        expect: () => const <AppearanceState>[
          AppearanceState(previewTextSize: KansoTextSize.larger),
        ],
        verify: (bloc) {
          expect(bloc.state.textSize, KansoTextSize.standard);
          expect(saved.previewTextSize, KansoTextSize.larger);
          expect(saved.textSize, KansoTextSize.standard);
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'diagramRendererChanged tells the core, shows it and saves it',
        build: build,
        act: (bloc) => bloc.add(
          const AppearanceEvent.diagramRendererChanged(DiagramRenderer.kanso),
        ),
        expect: () => const <AppearanceState>[
          AppearanceState(diagramRenderer: DiagramRenderer.kanso),
        ],
        verify: (_) {
          expect(themes.renderersSet, <DiagramRenderer>[DiagramRenderer.kanso]);
          expect(saved.diagramRenderer, DiagramRenderer.kanso);
        },
      );

      blocTest<AppearanceBloc, AppearanceState>(
        'started tells the core the saved diagram renderer',
        setUp: () {
          saved = const Preferences(
            previewTextSize: KansoTextSize.large,
            diagramRenderer: DiagramRenderer.kanso,
          );
        },
        build: build,
        act: (bloc) => bloc.add(const AppearanceEvent.started()),
        expect: () => const <AppearanceState>[
          AppearanceState(
            previewTextSize: KansoTextSize.large,
            diagramRenderer: DiagramRenderer.kanso,
            status: AppearanceStatus.ready,
          ),
        ],
        verify: (_) => expect(themes.renderersSet, <DiagramRenderer>[
          DiagramRenderer.kanso,
        ]),
      );
    });
  });
}
