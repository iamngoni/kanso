// Kanso — note_sharing_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/sharing/views/widgets/share_invite_bar.dart';
import 'package:kanso/sharing/views/widgets/share_note_button.dart';
import 'package:kanso/sharing/views/widgets/share_note_sheet.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/sharing_fixtures.dart';

Future<BrowseBlocs> _pumpNote(
  WidgetTester tester,
  String noteId,
  FixtureSharedNotesRepository shared, {
  FixtureLibraryRepository? library,
}) async {
  final blocs = await BrowseBlocs.start(library, null, shared);
  registerPageBlocs(blocs.library, renderer: blocs.renderer, shared: shared);
  final appearance = SeededAppearanceBloc(const AppearanceState());
  addTearDown(appearance.close);
  await atMockNow(
    () => pumpRouted(
      tester,
      BlocProvider<AppearanceBloc>.value(
        value: appearance,
        child: blocs.provide(NotePage(noteId: noteId)),
      ),
    ),
  );
  for (var i = 0; i < 4; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
  return blocs;
}

final Finder button = find.descendant(
  of: find.byType(ShareNoteButton),
  matching: find.byType(KansoIconButton),
);

void main() {
  group('A note’s share button', () {
    testWidgets('opens the share sheet of that note', (tester) async {
      final shared = FixtureSharedNotesRepository();
      await _pumpNote(tester, MockIds.syncDesign, shared);
      expect(find.byType(ShareNoteButton), findsOneWidget);
      expect(find.byType(ShareNoteSheet), findsNothing);
      await tester.tap(find.byType(ShareNoteButton));
      await tester.pumpAndSettle();
      expect(find.byType(ShareNoteSheet), findsOneWidget);
      expect(find.byType(ShareInviteBar), findsOneWidget);
      expect(shared.calls, contains('info ${MockIds.syncDesign}'));

      await tester.tap(find.bySemanticsLabel('Close'));
      await tester.pumpAndSettle();
      expect(find.byType(ShareNoteSheet), findsNothing);
    });

    testWidgets('is on a locked note too, whose sheet says it can not be '
        'shared', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..locked.add(MockIds.privateJournal);
      await _pumpNote(tester, MockIds.privateJournal, shared);
      await tester.tap(find.byType(ShareNoteButton));
      await tester.pumpAndSettle();
      expect(
        find.text(
          'Locked notes, and notes in locked notebooks, can’t be shared. '
          'Remove the lock first.',
        ),
        findsOneWidget,
      );
      expect(find.byType(ShareInviteBar), findsNothing);
    });

    testWidgets('is not there while the note is read', (tester) async {
      final library = FixtureLibraryRepository()..noteGate = Completer<void>();
      await _pumpNote(
        tester,
        MockIds.syncDesign,
        FixtureSharedNotesRepository(),
        library: library,
      );
      expect(button, findsNothing);
      library.noteGate!.complete();
      await tester.pump();
      await tester.pump();
      expect(button, findsOneWidget);
    });

    testWidgets('is not there when the note is gone', (tester) async {
      await _pumpNote(tester, 'note:gone', FixtureSharedNotesRepository());
      expect(button, findsNothing);
    });
  });

  group('A shared note’s live room', () {
    testWidgets('is joined when the note is opened and left when it is '
        'closed', (tester) async {
      final shared = FixtureSharedNotesRepository();
      shared.shares[MockIds.syncDesign] = shareInfo(MockIds.syncDesign);
      await _pumpNote(tester, MockIds.syncDesign, shared);
      expect(shared.live, <String>{MockIds.syncDesign});
      await tester.pumpWidget(const SizedBox.shrink());
      // The room is left once the bloc has closed, a few turns later.
      await tester.runAsync(() => Future<void>.delayed(Duration.zero));
      await tester.pump();
      expect(shared.live, isEmpty);
      expect(shared.calls, contains('closeLive ${MockIds.syncDesign}'));
    });

    testWidgets('is not opened for a note that is not shared', (tester) async {
      final shared = FixtureSharedNotesRepository();
      await _pumpNote(tester, MockIds.syncDesign, shared);
      expect(shared.live, isEmpty);
      expect(shared.calls.where((c) => c.startsWith('openLive')), isEmpty);
    });

    testWidgets('is not opened for a locked note', (tester) async {
      final shared = FixtureSharedNotesRepository()
        ..locked.add(MockIds.privateJournal);
      shared.shares[MockIds.privateJournal] = shareInfo(MockIds.privateJournal);
      await _pumpNote(tester, MockIds.privateJournal, shared);
      expect(shared.live, isEmpty);
    });

    testWidgets('is joined again when the app comes back to the front', (
      tester,
    ) async {
      final shared = FixtureSharedNotesRepository();
      shared.shares[MockIds.syncDesign] = shareInfo(MockIds.syncDesign);
      await _pumpNote(tester, MockIds.syncDesign, shared);
      shared.live.clear();
      for (final state in <AppLifecycleState>[
        AppLifecycleState.inactive,
        AppLifecycleState.hidden,
        AppLifecycleState.paused,
        AppLifecycleState.hidden,
        AppLifecycleState.inactive,
        AppLifecycleState.resumed,
      ]) {
        tester.binding.handleAppLifecycleStateChanged(state);
      }
      for (var i = 0; i < 4; i++) {
        await tester.pump(const Duration(milliseconds: 50));
      }
      expect(shared.live, <String>{MockIds.syncDesign});
    });
  });
}
