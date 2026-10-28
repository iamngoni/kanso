// Kanso — theme_gallery_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_search_input.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_gallery_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';

/// One state of the theme picker with its gallery, as a golden of the whole
/// window.
class _Screen {
  const _Screen(this.name, this.gallery, {this.interact});

  final String name;
  final FixtureGalleryRepository Function() gallery;

  /// What is done on the picker before the picture.
  final Future<void> Function(WidgetTester tester)? interact;
}

Future<void> _open(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

final List<_Screen> _screens = <_Screen>[
  const _Screen('theme_gallery', FixtureGalleryRepository.new),
  _Screen(
    'theme_gallery_scrolled',
    FixtureGalleryRepository.new,
    interact:
        (
          tester,
        ) async {
          await tester.drag(
            find.byType(Scrollable).first,
            const Offset(0, -420),
          );
          await tester.pumpAndSettle();
        },
  ),
  _Screen(
    'theme_gallery_loading',
    () => FixtureGalleryRepository()..gate = Completer<void>(),
  ),
  _Screen(
    'theme_gallery_failed',
    () =>
        FixtureGalleryRepository()
          ..cachedFailure = const ApplicationError.network(message: 'Off.'),
  ),
  _Screen(
    'theme_gallery_stale',
    () =>
        FixtureGalleryRepository()
          ..refreshFailure = const ApplicationError.network(message: 'Off.'),
  ),
  _Screen(
    'theme_gallery_no_results',
    FixtureGalleryRepository.new,
    interact:
        (
          tester,
        ) async {
          final hint = find.text('Search themes');
          await tester.scrollUntilVisible(
            hint,
            240,
            scrollable: find
                .descendant(
                  of: find.byType(ThemePickerPane),
                  matching: find.byType(Scrollable),
                )
                .first,
          );
          final field = find.ancestor(
            of: hint,
            matching: find.byType(KansoSearchInput),
          );
          await tester.enterText(
            find.descendant(of: field, matching: find.byType(EditableText)),
            'zzz',
          );
          await tester.pumpAndSettle();
        },
  ),
  _Screen(
    'theme_gallery_empty',
    () => FixtureGalleryRepository(items: <GalleryItem>[]),
  ),
];

void main() {
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final viewport in viewports) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in _screens) {
          testWidgets(screen.name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name: '${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              gallery: screen.gallery,
              interact: (tester, app) async {
                await _open(tester, app, '/settings');
                await _open(tester, app, '/settings/theme');
                await screen.interact?.call(tester);
              },
            );
          });
        }
      });
    }
  }
}
