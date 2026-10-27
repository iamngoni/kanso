// Kanso — icon_themes_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/painting.dart' show Size;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/settings/state/icon_themes/icon_themes_bloc.dart';
import 'package:kanso/settings/views/widgets/icon_theme_choice_row.dart';
import 'package:kanso/settings/views/widgets/icon_theme_preview_strip.dart';
import 'package:kanso/settings/views/widgets/icon_themes_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_gallery_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/real_icon_themes.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

class _Icons {
  _Icons(this.appearance, this.themes);

  final AppearanceBloc appearance;
  final IconThemesBloc themes;
}

Future<_Icons> _pump(
  WidgetTester tester, {
  MockRendererRepository? renderer,
}) async {
  final appearance = AppearanceBloc(
    ForgetfulPreferencesRepository(),
    RealIconThemesRenderer(),
    FixtureGalleryRepository(),
  );
  closeBlocLater(appearance);
  final themes = IconThemesBloc(renderer ?? RealIconThemesRenderer())
    ..add(const IconThemesEvent.started());
  closeBlocLater(themes);
  appearance.add(const AppearanceEvent.started());
  await tester.runAsync(() => Future<void>.delayed(Duration.zero));
  await pumpRouted(
    tester,
    MultiBlocProvider(
      providers: <BlocProvider<dynamic>>[
        BlocProvider<AppearanceBloc>.value(value: appearance),
        BlocProvider<IconThemesBloc>.value(value: themes),
      ],
      child: const IconThemesPane(),
    ),
    size: const Size(393, 4000),
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump();
  }
  return _Icons(appearance, themes);
}

void main() {
  loadRealIconThemes();

  group('IconThemesPane', () {
    testWidgets('lists every icon theme, Lucide first and in use', (
      tester,
    ) async {
      final screen = await _pump(tester);
      final rows = tester
          .widgetList<IconThemeChoiceRow>(find.byType(IconThemeChoiceRow))
          .toList();
      expect(
        rows.map((row) => row.theme.id).toList(),
        realIconThemes.map((theme) => theme.id).toList(),
      );
      expect(rows.first.theme.id, 'lucide');
      expect(rows.where((row) => row.selected).map((r) => r.theme.id), <String>[
        'lucide',
      ]);
      expect(find.text('Icons'), findsOneWidget);
      expect(find.text('Phosphor'), findsOneWidget);
      expect(find.byType(IconThemePreviewStrip), findsNWidgets(rows.length));
      expect(screen.appearance.state.iconSet.id, 'lucide');
    });

    testWidgets('choosing a theme marks it, shows it at once and keeps '
        'the others unmarked', (tester) async {
      final screen = await _pump(tester);
      await tester.tap(find.text('Phosphor'));
      await tester.pump();
      await tester.pump();
      expect(screen.appearance.state.iconSet.id, 'phosphor');
      expect(screen.appearance.state.iconSet.svgs, isNotEmpty);
      final rows = tester
          .widgetList<IconThemeChoiceRow>(find.byType(IconThemeChoiceRow))
          .toList();
      expect(rows.where((row) => row.selected).map((r) => r.theme.id), <String>[
        'phosphor',
      ]);
    });

    testWidgets('says who made a theme and under which licence', (
      tester,
    ) async {
      await _pump(tester);
      final phosphor = realIconThemes.firstWhere((t) => t.id == 'phosphor');
      expect(
        find.text('${phosphor.author}, ${phosphor.license}'),
        findsWidgets,
      );
    });

    testWidgets('shows still rows while the themes are read', (tester) async {
      final renderer = MockRendererRepository();
      final gate = Completer<Either<ApplicationError, List<IconThemeRow>>>();
      when(renderer.iconThemes).thenAnswer((_) => gate.future);
      await _pump(tester, renderer: renderer);
      expect(find.bySemanticsLabel('Loading icon themes'), findsOneWidget);
      expect(find.byType(IconThemeChoiceRow), findsNothing);
    });

    testWidgets('says plainly when they could not be read and tries again', (
      tester,
    ) async {
      final renderer = MockRendererRepository();
      var calls = 0;
      when(renderer.iconThemes).thenAnswer((_) async {
        calls++;
        return calls == 1
            ? const Left<ApplicationError, List<IconThemeRow>>(
                ApplicationError.internal(message: 'No icon themes.'),
              )
            : Right<ApplicationError, List<IconThemeRow>>(realIconThemes);
      });
      await _pump(tester, renderer: renderer);
      expect(find.text('The icon themes didn’t load'), findsOneWidget);
      expect(find.textContaining('No icon themes.'), findsNothing);
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      expect(find.byType(IconThemeChoiceRow), findsWidgets);
    });

    testWidgets('says so when there are no themes, and offers to try again', (
      tester,
    ) async {
      final renderer = MockRendererRepository();
      when(renderer.iconThemes).thenAnswer(
        (_) async => const Right<ApplicationError, List<IconThemeRow>>(
          <IconThemeRow>[],
        ),
      );
      await _pump(tester, renderer: renderer);
      expect(find.text('No icon themes'), findsOneWidget);
      expect(find.text('Try again'), findsOneWidget);
    });
  });
}
