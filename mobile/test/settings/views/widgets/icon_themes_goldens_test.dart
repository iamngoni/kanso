// Kanso — icon_themes_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_renderer_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_icon_themes.dart';
import '../../../helpers/real_themes.dart';

Future<void> _push(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

Future<void> _openIcons(WidgetTester tester, BrowseApp app) async {
  await _push(tester, app, '/settings');
  await _push(tester, app, '/settings/icons');
}

FixtureRendererRepository _real() => RealIconThemesFixtureRenderer();

AppearanceState _phosphor() =>
    AppearanceState(iconSet: realIconSet('phosphor'));

void main() {
  loadRealThemes();
  loadRealIconThemes();

  for (final viewport in <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ]) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        testWidgets("the Icons page, Kanso's own icons in use", (
          tester,
        ) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: '${viewport.slug}_settings_icons_${brightness.name}',
            viewport: viewport,
            brightness: brightness,
            renderer: _real,
            interact: _openIcons,
          );
        });

        testWidgets('the Icons page, Phosphor in use', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: '${viewport.slug}_settings_icons_phosphor_${brightness.name}',
            viewport: viewport,
            brightness: brightness,
            renderer: _real,
            appearance: _phosphor(),
            interact: _openIcons,
          );
        });

        testWidgets('settings drawn in Phosphor', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: '${viewport.slug}_settings_in_phosphor_${brightness.name}',
            viewport: viewport,
            brightness: brightness,
            renderer: _real,
            appearance: _phosphor(),
            interact: (tester, app) => _push(tester, app, '/settings'),
          );
        });

        testWidgets('the note list drawn in Phosphor', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: '${viewport.slug}_note_list_in_phosphor_${brightness.name}',
            viewport: viewport,
            brightness: brightness,
            renderer: _real,
            appearance: _phosphor(),
          );
        });
      });
    }
  }

  for (final brightness in Brightness.values) {
    group(
      'phone, ${brightness.name}, the Icons page while it loads or fails',
      () {
        testWidgets('loading', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: 'phone_settings_icons_loading_${brightness.name}',
            viewport: KansoViewport.phone,
            brightness: brightness,
            renderer: () =>
                FixtureRendererRepository()..iconThemesGate = Completer<void>(),
            interact: _openIcons,
          );
        });

        testWidgets('failed', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: 'phone_settings_icons_failed_${brightness.name}',
            viewport: KansoViewport.phone,
            brightness: brightness,
            renderer: () => FixtureRendererRepository()
              ..iconThemesResult =
                  const Left<ApplicationError, List<IconThemeRow>>(
                    ApplicationError.internal(message: 'No icon themes.'),
                  ),
            interact: _openIcons,
          );
        });

        testWidgets('empty', (tester) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: 'phone_settings_icons_empty_${brightness.name}',
            viewport: KansoViewport.phone,
            brightness: brightness,
            renderer: () => FixtureRendererRepository()
              ..iconThemesResult =
                  const Right<ApplicationError, List<IconThemeRow>>(
                    <IconThemeRow>[],
                  ),
            interact: _openIcons,
          );
        });
      },
    );
  }
}
