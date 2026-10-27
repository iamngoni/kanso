// Kanso — appearance_bloc_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

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
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/fixture_themes_repository.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_tokens.dart';

final KansoPalette _nord = fakePalette('nord', dark: true);
final KansoPalette _github = fakePalette('github-light');

/// [colors] without the code block's background, which belongs to the
/// preview theme and not to the syntax theme.
KansoSyntaxColors _tokensOnly(KansoSyntaxColors colors) =>
    colors.copyWith(codeBackground: const Color(0xFF000000));

void main() {
  late MockPreferencesRepository preferences;
  late FixtureThemesRepository themes;
  late Preferences saved;

  setUpAll(() {
    registerFallbackValue(const Preferences());
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    preferences = MockPreferencesRepository();
    themes = FixtureThemesRepository(
      themes: <InstalledTheme>[
        fixtureTheme(KansoPalette.kansoLight),
        fixtureTheme(KansoPalette.kansoDark),
        fixtureTheme(_nord),
        fixtureTheme(_github),
      ],
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
  });

  AppearanceBloc build() => AppearanceBloc(preferences, themes);

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
        expect(themes.fits, <(String, String, String)>[
          ('nord', 'nord', 'nord'),
        ]);
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
      'started with a theme that is gone shows the Kanso one and says failed',
      setUp: () {
        saved = const Preferences(
          themeSelection: ThemeSelection(light: 'gone'),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const AppearanceEvent.started()),
      expect: () => <AppearanceState>[
        AppearanceState(
          selection: saved.themeSelection,
          status: AppearanceStatus.failed,
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
