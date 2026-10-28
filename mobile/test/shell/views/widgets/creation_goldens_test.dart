// Kanso — creation_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/note_list/views/widgets/new_note_button.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';

/// One screen of making notes and notebooks, as a golden of the whole window.
class CreationScreen {
  const CreationScreen(
    this.name, {
    required this.viewports,
    this.library,
    this.interact,
  });

  final String name;

  /// The windows it is pictured in.
  final List<KansoViewport> viewports;
  final FixtureLibraryRepository Function()? library;
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;
}

/// The phone, and the three tablets the issue asks for in both orientations.
const List<KansoViewport> _everywhere = <KansoViewport>[
  KansoViewport.phone,
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
  KansoViewport.androidTabletPortrait,
];

/// Where the other screens' pictures stop: the step-5 ones are pictured in
/// the first four windows already.
const List<KansoViewport> _missing = <KansoViewport>[
  KansoViewport.androidTabletPortrait,
];

Finder _iconButton(String label) => find.byWidgetPredicate(
  (widget) => widget is KansoIconButton && widget.semanticLabel == label,
);

Future<void> _push(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

Future<void> _openLibrary(WidgetTester tester, BrowseApp app) async {
  if (app.layoutClass.value != LayoutClass.triple) {
    await tester.tap(find.bySemanticsLabel('Library'));
    await tester.pumpAndSettle();
  }
}

/// The sheet that asks for a notebook's name, with a name typed and the
/// keyboard up.
Future<void> _newNotebook(WidgetTester tester, BrowseApp app) async {
  await _openLibrary(tester, app);
  await tester.tap(_iconButton('New notebook'));
  await tester.pumpAndSettle();
  await tester.enterText(find.byType(EditableText).last, 'Recipes');
  final dpr = tester.view.devicePixelRatio;
  tester.view.viewInsets = FakeViewPadding(bottom: 330 * dpr);
  // The keyboard itself, as the editor's pictures draw it.
  tester
      .state<NavigatorState>(find.byType(Navigator).first)
      .overlay!
      .insert(
        OverlayEntry(
          builder: (context) => Positioned(
            left: 0,
            right: 0,
            bottom: 0,
            height: 330,
            child: ColoredBox(color: context.colors.selected),
          ),
        ),
      );
  await tester.pumpAndSettle();
}

/// A new note asked for in a locked notebook: the line that says to unlock
/// it, afloat on the list.
Future<void> _newNoteLocked(WidgetTester tester, BrowseApp app) async {
  await tester.tap(
    app.layoutClass.value == LayoutClass.single
        ? find.byType(NewNoteButton)
        : _iconButton('New note'),
  );
  await tester.pumpAndSettle();
}

final List<CreationScreen> _screens = <CreationScreen>[
  const CreationScreen('note_list', viewports: _missing),
  CreationScreen(
    'library_drawer',
    viewports: _missing,
    interact: (tester, app) async {
      await _openLibrary(tester, app);
    },
  ),
  CreationScreen(
    'settings',
    viewports: _missing,
    interact: (tester, app) => _push(tester, app, '/settings'),
  ),
  const CreationScreen(
    'new_notebook_sheet',
    viewports: _everywhere,
    interact: _newNotebook,
  ),
  CreationScreen(
    'settings_default_notebook',
    viewports: _everywhere,
    interact: (tester, app) async {
      await _push(tester, app, '/settings');
      await _push(tester, app, '/settings/notebook');
    },
  ),
  CreationScreen(
    'note_list_new_note_locked',
    viewports: _everywhere,
    library: () =>
        FixtureLibraryRepository()
          ..createResult = const Left<ApplicationError, Saved>(
            ApplicationError.locked(message: 'locked'),
          ),
    interact: _newNoteLocked,
  ),
];

void main() {
  loadRealThemes();

  for (final brightness in Brightness.values) {
    for (final screen in _screens) {
      for (final viewport in screen.viewports) {
        testWidgets('${viewport.slug}, ${brightness.name}, ${screen.name}', (
          tester,
        ) async {
          await expectAppGoldenOnBothPlatforms(
            tester,
            name: '${viewport.slug}_${screen.name}_${brightness.name}',
            viewport: viewport,
            brightness: brightness,
            library: screen.library,
            interact: screen.interact,
          );
        });
      }
    }
  }
}
