// Kanso — tab_bar_goldens_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/shell/views/widgets/list_note_layout.dart';
import 'package:kanso/shell/views/widgets/tablet_layout.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';
import 'package:kanso/tabs/views/widgets/tabbed_note.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/golden.dart';
import '../../helpers/kanso_viewport.dart';
import '../../helpers/library_fixtures.dart';

const BrowseLocation _allNotes = BrowseLocation.place(ListPlace.all());

/// The tablets a note's tabs are drawn on: an iPad in both ways round, and an
/// Android tablet.
const List<KansoViewport> _viewports = <KansoViewport>[
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
];

/// The three notes of the bar, in the order they are opened.
const List<String> _open = <String>[
  MockIds.syncDesign,
  MockIds.weeklyReview,
  MockIds.sourdough,
];

/// The panes a window of [viewport]'s width has room for, as the shell lays
/// them out: the library beside the other two when it fits, in its drawer
/// (so not drawn) when it does not.
Widget _panes(KansoViewport viewport) {
  const list = NoteListPane(
    openNoteId: MockIds.syncDesign,
    beside: true,
  );
  const note = TabbedNote(child: NotePane(inPane: true));
  final body = KansoLayout.classFor(viewport.size) == LayoutClass.triple
      ? const TabletLayout(
          library: LibraryPane(location: _allNotes),
          list: list,
          note: note,
        )
      : const ListNoteLayout(list: list, note: note);
  return Scaffold(resizeToAvoidBottomInset: false, body: body);
}

void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    for (final viewport in _viewports) {
      testWidgets('three tabs on ${viewport.slug}, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        // Every note of the mock list can be opened, under its own title.
        // The notes the fixture already knows stay: they carry the tags and
        // the status the screen shows.
        for (final entry in readableMockNotes().entries) {
          blocs.library.notes.putIfAbsent(entry.key, () => entry.value);
        }
        final list = await blocs.listOf(const ListPlace.all());
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        final tabs = blocs.tabs
          ..add(const TabsEvent.enabledChanged(enabled: true));
        // One after another, as the taps would arrive.
        for (final id in _open) {
          tabs.add(TabsEvent.openedInNewTab(id));
        }
        final opened = await tabs.stream
            .firstWhere((state) => state.tabs.length == _open.length)
            .timeout(const Duration(seconds: 10));
        expect(
          opened.tabs.open.map((tab) => tab.meta.id).toList(),
          _open,
          reason: 'each note of the bar is a tab, left to right',
        );
        // The first note is the one shown: the bar says so too.
        tabs.add(const TabsEvent.activated(0));
        final shown = await tabs.stream
            .firstWhere((state) => state.tabs.active == 0)
            .timeout(const Duration(seconds: 10));
        expect(shown.tabs.length, _open.length);

        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_panes(viewport), list: list, note: note),
            name: 'tablet_tabs_${viewport.slug}_$name',
            viewport: viewport,
            brightness: brightness,
          ),
        );
      });
    }
  }
}
