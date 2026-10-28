// Kanso — theme_gallery_section_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/renderer_repository.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/themes/state/theme_gallery/theme_gallery_bloc.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/views/widgets/gallery_theme_card.dart';
import 'package:kanso/themes/views/widgets/installed_theme_card.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_gallery_repository.dart';
import '../../../helpers/fixture_renderer_repository.dart';
import '../../../helpers/gallery_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

class _Picker {
  _Picker(this.appearance, this.picker, this.gallery);

  final AppearanceBloc appearance;
  final ThemePickerBloc picker;
  final ThemeGalleryBloc gallery;
}

/// A renderer with no built-in theme to find: every theme the appearance asks
/// of it is looked for in the gallery.
class _BuiltInlessRenderer extends Fake implements RendererRepository {
  @override
  Future<Either<ApplicationError, ThemeTokens>> builtinThemeTokens(
    String id,
  ) async => const Left<ApplicationError, ThemeTokens>(
    ApplicationError.notFound(message: 'Not built in.'),
  );
}

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 30));
  }
}

Future<_Picker> _pump(
  WidgetTester tester,
  FixtureGalleryRepository fixtures, {
  Size size = const Size(393, 2400),
}) async {
  final appearance = AppearanceBloc(
    ForgetfulPreferencesRepository(),
    _BuiltInlessRenderer(),
    fixtures,
  );
  closeBlocLater(appearance);
  final picker = ThemePickerBloc(FixtureRendererRepository(), fixtures)
    ..add(const ThemePickerEvent.started());
  closeBlocLater(picker);
  final gallery = ThemeGalleryBloc(fixtures)
    ..add(const ThemeGalleryEvent.started());
  closeBlocLater(gallery);
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
    size: size,
  );
  await _settle(tester);
  return _Picker(appearance, picker, gallery);
}

Finder _galleryCard(String name) => find.ancestor(
  of: find.descendant(
    of: find.byType(GalleryThemeCard),
    matching: find.text(name),
  ),
  matching: find.byType(GalleryThemeCard),
);

List<String> _galleryOrder(WidgetTester tester) => tester
    .widgetList<GalleryThemeCard>(find.byType(GalleryThemeCard))
    .map((card) => card.item.id)
    .toList();

List<String> _installedRow(WidgetTester tester) => tester
    .widgetList<InstalledThemeCard>(find.byType(InstalledThemeCard))
    .map((card) => card.theme.id)
    .toList();

Future<void> _press(WidgetTester tester, Finder button) async {
  await tester.ensureVisible(button);
  await tester.pump();
  await tester.tap(button);
  await _settle(tester);
}

Finder _in(String name, Finder what) =>
    find.descendant(of: _galleryCard(name), matching: what);

void main() {
  group('The gallery on the theme picker', () {
    testWidgets('shows the community’s themes as cards with what is known of '
        'each', (tester) async {
      await _pump(tester, FixtureGalleryRepository());
      expect(find.text('Gallery'), findsOneWidget);
      expect(find.text('Search themes'), findsOneWidget);
      expect(find.text('Popular'), findsOneWidget);
      expect(find.text('Recently updated'), findsOneWidget);
      expect(find.byType(GalleryThemeCard), findsNWidgets(4));

      expect(_in('Dusk', find.text('Official')), findsOneWidget);
      expect(_in('Dusk', find.text('by Kanso · Version 3')), findsOneWidget);
      expect(
        _in('Dusk', find.text('github.com/iamngoni/kanso-themes')),
        findsOneWidget,
      );
      expect(_in('Dusk', find.text('Install')), findsOneWidget);
      // A community theme says so.
      expect(_in('Moss', find.text('Community')), findsOneWidget);
    });

    testWidgets('does not show an install count while the catalog counts '
        'none, and shows it when it does', (tester) async {
      await _pump(
        tester,
        FixtureGalleryRepository(
          items: <GalleryItem>[
            galleryItem('quiet', name: 'Quiet'),
            galleryItem('busy', name: 'Busy', installs: 12),
          ],
        ),
      );
      expect(_in('Quiet', find.textContaining('install')), findsNothing);
      expect(
        _in('Busy', find.text('by Ana · Version 2 · 12 installs')),
        findsOneWidget,
      );
    });

    testWidgets('fetches the last index first and the current one behind it', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository();
      await _pump(tester, fixtures);
      expect(fixtures.indexCalls, <bool>[false, true]);
    });

    testWidgets('installed gallery themes are in the Installed row too, '
        'not marked as built in, and an icon theme is not', (tester) async {
      await _pump(tester, FixtureGalleryRepository());
      expect(_installedRow(tester), containsAll(<String>['paper', 'moss']));
      expect(_installedRow(tester), isNot(contains('lines')));
      expect(_installedRow(tester), isNot(contains('dusk')));
      final moss = tester
          .widgetList<InstalledThemeCard>(find.byType(InstalledThemeCard))
          .firstWhere((card) => card.theme.id == 'moss');
      expect(moss.theme.builtIn, isFalse);
      expect(
        find.descendant(
          of: find.byType(InstalledThemeCard),
          matching: find.text('Installed'),
        ),
        findsNWidgets(2),
      );
    });

    testWidgets('an icon theme is a tile with an Icons badge', (tester) async {
      await _pump(tester, FixtureGalleryRepository());
      expect(_in('Lines', find.text('Icons')), findsOneWidget);
      expect(
        tester.widget<GalleryThemeCard>(_galleryCard('Lines')).preview,
        isNull,
      );
    });

    testWidgets('installs a theme, which joins the Installed row', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository();
      await _pump(tester, fixtures);
      expect(_installedRow(tester), isNot(contains('dusk')));
      await _press(
        tester,
        _in('Dusk', find.widgetWithText(KansoButton, 'Install')),
      );
      expect(fixtures.installs, <String>['dusk']);
      expect(_installedRow(tester), contains('dusk'));
      expect(
        _in('Dusk', find.widgetWithText(KansoButton, 'Installed')),
        findsOneWidget,
      );
      expect(_in('Dusk', find.text('Remove')), findsOneWidget);
    });

    testWidgets('updates a theme that has a newer version', (tester) async {
      final fixtures = FixtureGalleryRepository();
      await _pump(tester, fixtures);
      final update = _in('Paper', find.widgetWithText(KansoButton, 'Update'));
      expect(update, findsOneWidget);
      await _press(tester, update);
      expect(fixtures.installs, <String>['paper']);
      expect(
        _in('Paper', find.widgetWithText(KansoButton, 'Installed')),
        findsOneWidget,
      );
    });

    testWidgets('removes an installed theme from the Installed row', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository();
      await _pump(tester, fixtures);
      await _press(
        tester,
        _in('Moss', find.widgetWithText(KansoButton, 'Remove')),
      );
      expect(fixtures.removals, <String>['moss']);
      expect(_installedRow(tester), isNot(contains('moss')));
      expect(
        _in('Moss', find.widgetWithText(KansoButton, 'Install')),
        findsOneWidget,
      );
    });

    testWidgets('a removed theme that was in use goes back to Kanso themes', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository();
      final picker = await _pump(tester, fixtures);
      final moss = find.ancestor(
        of: find.descendant(
          of: find.byType(InstalledThemeCard),
          matching: find.text('Moss'),
        ),
        matching: find.byType(InstalledThemeCard),
      );
      await tester.ensureVisible(moss);
      await tester.tap(moss);
      await _settle(tester);
      expect(picker.appearance.state.selection.light, 'moss');
      expect(picker.appearance.state.light.id, 'moss');

      await _press(
        tester,
        _in('Moss', find.widgetWithText(KansoButton, 'Remove')),
      );
      expect(picker.appearance.state.selection.light, 'kanso-light');
      expect(picker.appearance.state.light.id, 'kanso-light');
    });

    testWidgets('a theme that needs a newer Kanso cannot be installed', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository(
        items: <GalleryItem>[galleryItem('new', name: 'New', runsHere: false)],
      );
      await _pump(tester, fixtures);
      final button = find.widgetWithText(KansoButton, 'Needs a newer Kanso');
      expect(button, findsOneWidget);
      expect(tester.widget<KansoButton>(button).onPressed, isNull);
    });

    testWidgets('an install that fails says why and changes nothing', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository()
        ..installFailure = const ApplicationError.invalid(
          message: 'Dusk didn’t match the catalog: wrong size.',
        );
      await _pump(tester, fixtures);
      await _press(
        tester,
        _in('Dusk', find.widgetWithText(KansoButton, 'Install')),
      );
      expect(
        find.text('Dusk didn’t match the catalog: wrong size.'),
        findsOneWidget,
      );
      expect(_installedRow(tester), isNot(contains('dusk')));
      expect(
        _in('Dusk', find.widgetWithText(KansoButton, 'Install')),
        findsOneWidget,
      );
    });

    testWidgets('shows a button as busy while its theme is installing', (
      tester,
    ) async {
      final gate = Completer<void>();
      addTearDown(() {
        if (!gate.isCompleted) {
          gate.complete();
        }
      });
      final fixtures = FixtureGalleryRepository()..installGate = gate;
      final picker = await _pump(tester, fixtures);
      picker.gallery.add(const ThemeGalleryEvent.installed('dusk'));
      await _settle(tester);
      expect(_in('Dusk', find.text('Installing…')), findsOneWidget);
      gate.complete();
      await _settle(tester);
      expect(find.text('Installing…'), findsNothing);
      expect(_in('Dusk', find.text('Installed')), findsOneWidget);
    });
  });

  group('Searching and ordering the gallery', () {
    testWidgets('a search narrows the cards', (tester) async {
      await _pump(tester, FixtureGalleryRepository());
      await tester.ensureVisible(find.byType(EditableText));
      await tester.enterText(find.byType(EditableText), 'dusk');
      await _settle(tester);
      expect(_galleryOrder(tester), <String>['dusk']);
      await tester.enterText(find.byType(EditableText), '');
      await _settle(tester);
      expect(_galleryOrder(tester).length, 4);
    });

    testWidgets('a search with no match says so with the words typed', (
      tester,
    ) async {
      await _pump(tester, FixtureGalleryRepository());
      await tester.ensureVisible(find.byType(EditableText));
      await tester.enterText(find.byType(EditableText), 'zzz');
      await _settle(tester);
      expect(find.byType(GalleryThemeCard), findsNothing);
      expect(find.text('No themes match'), findsOneWidget);
      expect(
        find.text('Nothing in the gallery matches “zzz”. Try another word.'),
        findsOneWidget,
      );
    });

    testWidgets('Popular puts Kanso first and Recently updated the newest', (
      tester,
    ) async {
      await _pump(tester, FixtureGalleryRepository());
      expect(_galleryOrder(tester).first, 'dusk');
      await tester.ensureVisible(find.text('Recently updated'));
      await tester.tap(find.text('Recently updated'));
      await _settle(tester);
      // Paper was updated last.
      expect(_galleryOrder(tester).first, 'paper');
      await tester.tap(find.text('Popular'));
      await _settle(tester);
      expect(_galleryOrder(tester).first, 'dusk');
    });

    testWidgets('an empty gallery says so', (tester) async {
      await _pump(tester, FixtureGalleryRepository(items: <GalleryItem>[]));
      expect(find.text('The gallery is empty'), findsOneWidget);
      expect(find.byType(GalleryThemeCard), findsNothing);
    });
  });

  group('The gallery while it loads and when it fails', () {
    testWidgets('shows still cards while the index is read, and says so', (
      tester,
    ) async {
      final gate = Completer<void>();
      addTearDown(() {
        if (!gate.isCompleted) {
          gate.complete();
        }
      });
      final handle = tester.ensureSemantics();
      await _pump(tester, FixtureGalleryRepository()..gate = gate);
      expect(find.bySemanticsLabel('Loading the gallery'), findsOneWidget);
      expect(find.byType(GalleryThemeCard), findsNothing);
      // The installed themes do not wait for it.
      expect(_installedRow(tester), contains('moss'));
      handle.dispose();
    });

    testWidgets('says plainly when no index can be read, and tries again', (
      tester,
    ) async {
      final fixtures = FixtureGalleryRepository()
        ..cachedFailure = const ApplicationError.network(message: 'dns');
      await _pump(tester, fixtures);
      expect(find.text('The gallery didn’t load'), findsOneWidget);
      expect(find.textContaining('dns'), findsNothing);
      fixtures.cachedFailure = null;
      await _press(tester, find.text('Try again').last);
      expect(find.text('The gallery didn’t load'), findsNothing);
      expect(find.byType(GalleryThemeCard), findsNWidgets(4));
    });

    testWidgets('keeps the last list when the current one cannot be '
        'fetched, and fetches it again when asked', (tester) async {
      final fixtures = FixtureGalleryRepository()
        ..refreshFailure = const ApplicationError.network(message: 'dns');
      await _pump(tester, fixtures);
      expect(find.byType(GalleryThemeCard), findsNWidgets(4));
      expect(
        find.text(
          'Showing the last list. Couldn’t reach the gallery just now.',
        ),
        findsOneWidget,
      );
      fixtures.refreshFailure = null;
      await _press(tester, find.text('Try again'));
      expect(find.textContaining('Showing the last list'), findsNothing);
    });
  });
}
