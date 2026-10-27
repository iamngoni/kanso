// Kanso — theme_picker_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/installed_theme.dart';
import 'package:kanso/core/models/data/installed_theme_origin.dart';
import 'package:kanso/core/models/data/theme_file_notice.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_check_badge.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/views/widgets/installed_theme_card.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_gallery_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_picker_blocs.dart';

class _Picker {
  _Picker(this.appearance, this.picker, this.gallery);

  final AppearanceBloc appearance;
  final ThemePickerBloc picker;
  final ThemeGalleryBloc gallery;
}

List<InstalledTheme> _themes() => <InstalledTheme>[
  fixtureTheme(KansoPalette.kansoLight),
  fixtureTheme(KansoPalette.kansoDark),
  fixtureTheme(fakePalette('nord', dark: true)),
  fixtureTheme(
    fakePalette('mine').copyWith(name: 'Mine'),
    origin: InstalledThemeOrigin.own,
  ),
  fixtureTheme(
    fakePalette('fresh').copyWith(name: 'Fresh'),
    origin: InstalledThemeOrigin.gallery,
  ),
];

Future<_Picker> _pump(
  WidgetTester tester, {
  MockRendererRepository? renderer,
  FixtureGalleryRepository? themes,
}) async {
  final fixtures = themes ?? emptyGallery();
  final appearance = AppearanceBloc(
    ForgetfulPreferencesRepository(),
    RealThemesRenderer(),
    fixtures,
  );
  closeBlocLater(appearance);
  final picker = ThemePickerBloc(renderer ?? RealThemesRenderer(), fixtures)
    ..add(const ThemePickerEvent.started());
  closeBlocLater(picker);
  final gallery = ThemeGalleryBloc(fixtures)
    ..add(const ThemeGalleryEvent.started());
  closeBlocLater(gallery);
  appearance.add(const AppearanceEvent.started());
  await pumpRouted(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<ThemePickerBloc>.value(value: picker),
        BlocProvider<ThemeGalleryBloc>.value(value: gallery),
      ],
      child: const ThemePickerPane(),
    ),
    size: const Size(393, 1700),
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump();
  }
  return _Picker(appearance, picker, gallery);
}

Finder _cards(ThemeCardKind kind) => find.byWidgetPredicate(
  (widget) => widget is InstalledThemeCard && widget.kind == kind,
  description: '$kind cards',
);

Finder _card(ThemeCardKind kind, String id) =>
    find.byKey(ValueKey<String>('${kind.name}-$id'));

Finder _same(ThemeCardKind kind) =>
    find.byKey(ValueKey<String>('same-${kind.name}'));

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 5; i++) {
    await tester.pump();
  }
}

void main() {
  group('ThemePickerPane', () {
    testWidgets('has the desktop’s three sections and their words', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Theme'), findsOneWidget);
      expect(
        find.text(
          'Colors the sidebar, the note list, menus and every control.',
        ),
        findsOneWidget,
      );
      // The gallery is below them, empty here.
      expect(find.text('Gallery'), findsOneWidget);
      expect(find.text('Search themes'), findsOneWidget);
      expect(find.text('The gallery is empty'), findsOneWidget);
      expect(find.text('Install'), findsNothing);
    });

    testWidgets('outlines both themes in use first while the system is '
        'followed, and Same as theme in the other sections', (tester) async {
      await _pump(tester);
      final cards = tester
          .widgetList<InstalledThemeCard>(_cards(ThemeCardKind.ui))
          .toList();
      expect(cards.first.theme!.id, 'kanso-light');
      expect(cards[1].theme!.id, 'kanso-dark');
      expect(
        cards.where((card) => card.current).map((c) => c.theme!.id),
        <String>['kanso-light', 'kanso-dark'],
      );
      expect(
        tester.widget<InstalledThemeCard>(_same(ThemeCardKind.syntax)).current,
        isTrue,
      );
      expect(
        tester.widget<InstalledThemeCard>(_same(ThemeCardKind.preview)).current,
        isTrue,
      );
      // Two in the interface row, one in each of the others.
      expect(find.byType(KansoCheckBadge), findsNWidgets(4));
    });

    testWidgets('has the switch with the desktop’s words', (tester) async {
      final screen = await _pump(tester);
      expect(
        find.text('Switch between a light and a dark theme with the system'),
        findsOneWidget,
      );
      expect(
        find.text(
          'Showing the light theme in light mode and the dark one in dark '
          'mode.',
        ),
        findsOneWidget,
      );
      expect(
        tester.widget<KansoSwitch>(find.byType(KansoSwitch)).value,
        isTrue,
      );

      await tester.tap(find.byType(KansoSwitch));
      await _settle(tester);
      expect(screen.appearance.state.selection.followSystem, isFalse);
      expect(
        find.textContaining('Off: the theme stays as picked'),
        findsOneWidget,
      );
    });

    testWidgets('choosing a card in the first section picks the interface '
        'theme and outlines it first', (tester) async {
      final screen = await _pump(tester);
      await tester.ensureVisible(_card(ThemeCardKind.ui, 'nord'));
      await tester.tap(_card(ThemeCardKind.ui, 'nord'));
      await _settle(tester);
      expect(screen.appearance.state.selection.dark, 'nord');
      expect(screen.appearance.state.dark.id, 'nord');
      final cards = tester
          .widgetList<InstalledThemeCard>(_cards(ThemeCardKind.ui))
          .toList();
      expect(
        cards.where((card) => card.current).map((c) => c.theme!.id).toSet(),
        <String>{'kanso-light', 'nord'},
      );
      expect(cards.first.current, isTrue);
    });

    testWidgets('choosing an Editor syntax card picks only the syntax theme, '
        'and Same as theme takes it back', (tester) async {
      final screen = await _pump(tester);
      await tester.ensureVisible(_card(ThemeCardKind.syntax, 'nord'));
      await tester.tap(_card(ThemeCardKind.syntax, 'nord'));
      await _settle(tester);
      var selection = screen.appearance.state.selection;
      expect(selection.syntax, 'nord');
      expect(selection.preview, isNull);
      expect(selection.light, 'kanso-light');
      expect(
        tester
            .widget<InstalledThemeCard>(_card(ThemeCardKind.syntax, 'nord'))
            .current,
        isTrue,
      );
      expect(
        tester.widget<InstalledThemeCard>(_same(ThemeCardKind.syntax)).current,
        isFalse,
      );
      // The interface row is untouched.
      expect(
        tester
            .widgetList<InstalledThemeCard>(_cards(ThemeCardKind.ui))
            .where((card) => card.current)
            .length,
        2,
      );

      await tester.ensureVisible(_same(ThemeCardKind.syntax));
      await tester.tap(_same(ThemeCardKind.syntax));
      await _settle(tester);
      selection = screen.appearance.state.selection;
      expect(selection.syntax, isNull);
      expect(
        tester.widget<InstalledThemeCard>(_same(ThemeCardKind.syntax)).current,
        isTrue,
      );
    });

    testWidgets('choosing a Preview card picks only the preview theme', (
      tester,
    ) async {
      final screen = await _pump(tester);
      await tester.ensureVisible(_card(ThemeCardKind.preview, 'mine'));
      await tester.tap(_card(ThemeCardKind.preview, 'mine'));
      await _settle(tester);
      final selection = screen.appearance.state.selection;
      expect(selection.preview, 'mine');
      expect(selection.syntax, isNull);
      expect(
        screen.appearance.state.light.preview,
        fakePalette('mine').preview,
      );
      expect(
        tester
            .widget<InstalledThemeCard>(_card(ThemeCardKind.preview, 'mine'))
            .current,
        isTrue,
      );
    });

    testWidgets('shows still cards while the themes are read', (tester) async {
      final gate = Completer<void>();
      final themes = FixtureThemesRepository(themes: _themes())
        ..listGate = gate.future;
      await _pump(tester, themes: themes);
      expect(find.bySemanticsLabel('Loading themes'), findsNWidgets(3));
      expect(find.byType(InstalledThemeCard), findsNothing);
      // The files section is there already.
      expect(find.text('Import theme'), findsOneWidget);
      gate.complete();
      await _settle(tester);
      expect(find.byType(InstalledThemeCard), findsWidgets);
    });

    testWidgets('says plainly when they could not be read and tries again', (
      tester,
    ) async {
      final themes = FixtureThemesRepository(themes: _themes())
        ..listFailure = const ApplicationError.internal(message: 'No themes.');
      await _pump(tester, themes: themes);
      expect(find.text('The themes didn’t load'), findsOneWidget);
      expect(find.textContaining('No themes'), findsNothing);
      themes.listFailure = null;
      await tester.tap(find.text('Try again'));
      await _settle(tester);
      expect(find.byType(InstalledThemeCard), findsWidgets);
    });

    group('Theme files', () {
      testWidgets('has the Import theme row with the desktop’s words', (
        tester,
      ) async {
        await _pump(tester);
        expect(find.text('Import theme'), findsOneWidget);
        expect(
          find.text(
            'A theme package (its folder or .tgz) or a Zed theme (.json), '
            'converted into the themes folder.',
          ),
          findsOneWidget,
        );
        expect(find.text('Import Theme…'), findsOneWidget);
      });

      testWidgets('lists a warning row for each file that was left out', (
        tester,
      ) async {
        await _pump(
          tester,
          themes: FixtureThemesRepository(
            themes: _themes(),
            themeNotices: const <ThemeFileNotice>[
              ThemeFileNotice(file: 'broken.json', sentence: 'Isn’t a theme.'),
              ThemeFileNotice(file: 'odd.tgz', sentence: 'Has no theme in it.'),
            ],
          ),
        );
        expect(find.text('broken.json: Isn’t a theme.'), findsOneWidget);
        expect(find.text('odd.tgz: Has no theme in it.'), findsOneWidget);
      });

      testWidgets('importing a file adds its theme and says so', (
        tester,
      ) async {
        final screen = await _pump(
          tester,
          files: FixtureFilePickerRepository(<String?>['/tmp/solarized.tgz']),
        );
        await tester.ensureVisible(find.text('Import Theme…'));
        await tester.tap(find.text('Import Theme…'));
        await _settle(tester);
        expect(screen.themes.importedThemePaths, <String>[
          '/tmp/solarized.tgz',
        ]);
        expect(find.text('Imported Solarized.'), findsOneWidget);
        expect(_card(ThemeCardKind.ui, 'solarized'), findsOneWidget);
        expect(_card(ThemeCardKind.syntax, 'solarized'), findsOneWidget);
        expect(_card(ThemeCardKind.preview, 'solarized'), findsOneWidget);
      });

      testWidgets('tapping the row imports too', (tester) async {
        final screen = await _pump(
          tester,
          files: FixtureFilePickerRepository(<String?>['/tmp/a.json']),
        );
        await tester.ensureVisible(find.text('Import theme'));
        await tester.tap(find.text('Import theme'));
        await _settle(tester);
        expect(screen.themes.importedThemePaths, <String>['/tmp/a.json']);
      });

      testWidgets('shows why an import failed', (tester) async {
        final themes = FixtureThemesRepository(themes: _themes())
          ..importFailure = const ApplicationError.invalid(
            message: 'That isn’t a Zed theme.',
          );
        await _pump(
          tester,
          themes: themes,
          files: FixtureFilePickerRepository(<String?>['/tmp/bad.json']),
        );
        await tester.ensureVisible(find.text('Import Theme…'));
        await tester.tap(find.text('Import Theme…'));
        await _settle(tester);
        expect(
          find.text('Couldn’t import the theme. That isn’t a Zed theme.'),
          findsOneWidget,
        );
        expect(find.textContaining('Imported'), findsNothing);
      });

      testWidgets('does nothing when the picker is closed', (tester) async {
        final screen = await _pump(tester);
        await tester.ensureVisible(find.text('Import Theme…'));
        await tester.tap(find.text('Import Theme…'));
        await _settle(tester);
        expect(screen.files.asked, 1);
        expect(screen.themes.importedThemePaths, isEmpty);
        expect(
          find.byKey(const ValueKey<String>('theme-import-outcome')),
          findsNothing,
        );
      });
    });
  });
}
