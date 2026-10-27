// Kanso — shell_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/router/kanso_sheet_page.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';
import 'package:kanso/shell/views/widgets/library_drawer_host.dart';
import 'package:kanso/shell/views/widgets/list_note_layout.dart';
import 'package:kanso/shell/views/widgets/tablet_layout.dart';
import 'package:kanso/themes/state/theme_picker/theme_picker_bloc.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/settings_blocs.dart';
import '../../../helpers/theme_picker_blocs.dart';

const BrowseLocation _allNotes = BrowseLocation.place(ListPlace.all());

Widget _scaffold(Widget child) {
  return Scaffold(resizeToAvoidBottomInset: false, body: child);
}

/// The three panes of a tablet, as the sheets of settings and themes float
/// over them.
Widget _threePanes() {
  return _scaffold(
    const TabletLayout(
      library: LibraryPane(location: _allNotes),
      list: NoteListPane(openNoteId: MockIds.syncDesign, beside: true),
      note: NotePane(inPane: true),
    ),
  );
}

/// Opens [sheet] over the page as the app's own sheet route does. The route
/// is above the page, so the appearance the page reads and the [providers]
/// the sheet needs are given to it again.
Future<void> _openSheet(
  WidgetTester tester, {
  required Widget sheet,
  List<BlocProvider<dynamic>> providers = const <BlocProvider<dynamic>>[],
}) async {
  final context = tester.element(find.byType(TabletLayout));
  final route = KansoSheetRoute<void>(
    page: KansoSheetPage<void>(
      child: MultiBlocProvider(
        providers: providers,
        child: BlocProvider<AppearanceBloc>.value(
          value: context.read<AppearanceBloc>(),
          child: sheet,
        ),
      ),
    ),
  );
  unawaited(Navigator.of(context).push(route));
  await tester.pumpAndSettle();
}

void main() {
  loadRealThemes();

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the shell pictures, $name', () {
      testWidgets('the library drawer over the list on a phone', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              LibraryDrawerHost(
                location: _allNotes,
                width: phoneSize.width * KansoSizes.drawerFraction,
                edgeDragEnabled: true,
                child: _scaffold(const NoteListPane()),
              ),
              list: list,
            ),
            name: 'phone_library_drawer_$name',
            size: phoneSize,
            brightness: brightness,
            interact: (tester) async {
              await tester.tap(find.bySemanticsLabel('Library'));
            },
          ),
        );
      });

      testWidgets('the library drawer over two panes on a tablet', (
        tester,
      ) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              LibraryDrawerHost(
                location: _allNotes,
                width: KansoSizes.libraryPane,
                edgeDragEnabled: true,
                child: _scaffold(
                  const ListNoteLayout(
                    list: NoteListPane(
                      openNoteId: MockIds.syncDesign,
                      beside: true,
                    ),
                    note: NotePane(inPane: true),
                  ),
                ),
              ),
              list: list,
              note: note,
            ),
            name: 'tablet_portrait_library_drawer_$name',
            size: tabletPortraitSize,
            brightness: brightness,
            interact: (tester) async {
              await tester.tap(find.bySemanticsLabel('Library'));
            },
          ),
        );
      });

      testWidgets('the settings sheet over three panes', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        final settings = await SettingsBlocs.start();
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_threePanes(), list: list, note: note),
            name: 'tablet_settings_sheet_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
            interact: (tester) => _openSheet(
              tester,
              sheet: const SettingsPane(),
              providers: settings.providers,
            ),
          ),
        );
      });

      testWidgets('the theme picker sheet over three panes', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        final picker = await startThemePicker();
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_threePanes(), list: list, note: note),
            name: 'tablet_theme_picker_sheet_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
            interact: (tester) => _openSheet(
              tester,
              sheet: const ThemePickerPane(),
              providers: <BlocProvider<dynamic>>[
                BlocProvider<ThemePickerBloc>.value(value: picker),
              ],
            ),
          ),
        );
      });
    });
  }
}
