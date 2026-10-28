// Kanso — theme_picker_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_check_badge.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/views/widgets/installed_theme_card.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_gallery_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_picker_blocs.dart';

class _Picker {
  _Picker(this.appearance, this.picker, this.gallery);

  final AppearanceBloc appearance;
  final ThemePickerBloc picker;
  final ThemeGalleryBloc gallery;
}

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
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
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
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump();
  }
  return _Picker(appearance, picker, gallery);
}

Finder _card(String name) => find.ancestor(
  of: find.text(name),
  matching: find.byType(InstalledThemeCard),
);

void main() {
  loadRealThemes();

  group('ThemePickerPane', () {
    testWidgets('shows a card for each built-in theme, in a row', (
      tester,
    ) async {
      await _pump(tester);
      expect(find.text('Installed'), findsOneWidget);
      expect(
        find.byType(InstalledThemeCard, skipOffstage: false),
        findsNWidgets(realThemes.length),
      );
      // The gallery is below them, empty here.
      expect(find.text('Gallery'), findsOneWidget);
      expect(find.text('Search themes'), findsOneWidget);
      expect(find.text('The gallery is empty'), findsOneWidget);
      expect(find.text('Install'), findsNothing);
    });

    testWidgets('outlines both themes in use first while the system is '
        'followed', (tester) async {
      await _pump(tester);
      final cards = tester
          .widgetList<InstalledThemeCard>(find.byType(InstalledThemeCard))
          .toList();
      expect(cards.first.theme.id, 'kanso-light');
      expect(cards[1].theme.id, 'kanso-dark');
      expect(cards.where((card) => card.current).map((c) => c.theme.id), [
        'kanso-light',
        'kanso-dark',
      ]);
      expect(find.byType(KansoCheckBadge), findsNWidgets(2));
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
      await tester.pump();
      await tester.pump();
      expect(screen.appearance.state.selection.followSystem, isFalse);
      expect(
        find.textContaining('Off: the theme stays as picked'),
        findsOneWidget,
      );
      expect(
        tester.widget<KansoSwitch>(find.byType(KansoSwitch)).value,
        isFalse,
      );
    });

    testWidgets('choosing a card picks the theme and outlines it first', (
      tester,
    ) async {
      final screen = await _pump(tester);
      await tester.ensureVisible(_card('Nord').first);
      await tester.tap(_card('Nord').first);
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(screen.appearance.state.selection.dark, 'nord');
      expect(screen.appearance.state.dark.id, 'nord');
      final cards = tester
          .widgetList<InstalledThemeCard>(find.byType(InstalledThemeCard))
          .toList();
      expect(
        cards.where((card) => card.current).map((c) => c.theme.id).toSet(),
        <String>{'kanso-light', 'nord'},
      );
      expect(cards.first.current, isTrue);
    });

    testWidgets('shows still cards while the themes are read', (tester) async {
      final renderer = MockRendererRepository();
      final gate = Completer<Either<ApplicationError, List<BuiltinTheme>>>();
      when(renderer.builtinThemes).thenAnswer((_) => gate.future);
      await _pump(tester, renderer: renderer);
      expect(find.bySemanticsLabel('Loading themes'), findsOneWidget);
      expect(find.byType(InstalledThemeCard), findsNothing);
    });

    testWidgets('says plainly when they could not be read and tries again', (
      tester,
    ) async {
      final renderer = MockRendererRepository();
      var calls = 0;
      when(renderer.builtinThemes).thenAnswer((_) async {
        calls++;
        return calls == 1
            ? const Left<ApplicationError, List<BuiltinTheme>>(
                ApplicationError.internal(message: 'No themes.'),
              )
            : Right<ApplicationError, List<BuiltinTheme>>(realThemes);
      });
      await _pump(tester, renderer: renderer);
      expect(find.text('The themes didn’t load'), findsOneWidget);
      expect(find.textContaining('No themes'), findsNothing);
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      expect(find.byType(InstalledThemeCard), findsWidgets);
    });
  });
}
