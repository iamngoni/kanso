// Kanso — note_pane_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/note/views/widgets/no_note_selected_pane.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/shell/views/widgets/list_note_layout.dart';
import 'package:kanso/shell/views/widgets/tablet_layout.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';

Widget _scaffold(Widget child) {
  return Scaffold(resizeToAvoidBottomInset: false, body: child);
}

void main() {
  group('a note on a phone', () {
    for (final brightness in Brightness.values) {
      final name = brightness.name;

      testWidgets('open, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(
          MockIds.syncDesign,
          themeId: 'kanso-$name',
        );
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(_scaffold(const NotePane(inPane: false)), note: note),
            name: 'phone_note_$name',
            size: phoneSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('locked, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(MockIds.privateJournal);
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NotePane(inPane: false)), note: note),
          name: 'phone_note_locked_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('not found, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf('note:gone');
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NotePane(inPane: false)), note: note),
          name: 'phone_note_not_found_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('while it loads, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final note = await blocs.noteOf(MockIds.syncDesign, read: false);
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NotePane(inPane: false)), note: note),
          name: 'phone_note_loading_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('after a failure, $name', (tester) async {
        final library = _FailingLibrary();
        final blocs = await BrowseBlocs.start(library);
        final note = await blocs.noteOf(MockIds.syncDesign);
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(_scaffold(const NotePane(inPane: false)), note: note),
          name: 'phone_note_failed_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });
    }
  });

  group('a note beside its list', () {
    for (final brightness in Brightness.values) {
      final name = brightness.name;

      testWidgets('three panes, $name', (tester) async {
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
              _scaffold(
                const TabletLayout(
                  library: LibraryPane(
                    location: BrowseLocation.place(ListPlace.all()),
                  ),
                  list: NoteListPane(
                    openNoteId: MockIds.syncDesign,
                    beside: true,
                  ),
                  note: NotePane(inPane: true),
                ),
              ),
              list: list,
              note: note,
            ),
            name: 'tablet_list_and_note_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('two panes, $name', (tester) async {
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
              _scaffold(
                const ListNoteLayout(
                  list: NoteListPane(
                    openNoteId: MockIds.syncDesign,
                    beside: true,
                  ),
                  note: NotePane(inPane: true),
                ),
              ),
              list: list,
              note: note,
            ),
            name: 'tablet_portrait_list_and_note_$name',
            size: tabletPortraitSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('a locked note, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        final note = await blocs.noteOf(MockIds.privateJournal);
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              _scaffold(
                const TabletLayout(
                  library: LibraryPane(
                    location: BrowseLocation.place(ListPlace.all()),
                  ),
                  list: NoteListPane(
                    openNoteId: MockIds.privateJournal,
                    beside: true,
                  ),
                  note: NotePane(inPane: true),
                ),
              ),
              list: list,
              note: note,
            ),
            name: 'tablet_locked_note_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
          ),
        );
      });

      testWidgets('no note open, $name', (tester) async {
        final blocs = await BrowseBlocs.start();
        final list = await blocs.listOf(const ListPlace.all());
        await atMockNow(
          () => expectGoldenOnBothPlatforms(
            tester,
            blocs.provide(
              _scaffold(
                const TabletLayout(
                  library: LibraryPane(
                    location: BrowseLocation.place(ListPlace.all()),
                  ),
                  list: NoteListPane(beside: true),
                  note: NoNoteSelectedPane(),
                ),
              ),
              list: list,
            ),
            name: 'tablet_no_note_selected_$name',
            size: tabletLandscapeSize,
            brightness: brightness,
          ),
        );
      });
    }
  });
}

class _FailingLibrary extends FixtureLibraryRepository {
  @override
  Future<Either<ApplicationError, NoteView>> note(String id) async =>
      const Left<ApplicationError, NoteView>(
        ApplicationError.storage(message: 'Disk error.'),
      );
}
