// Kanso — undo_notice_host_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_undo_notice.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';

Either<ApplicationError, Changed> _changed(int ticket, String message) =>
    Right<ApplicationError, Changed>(
      Changed(count: 1, undo: ticket, message: message),
    );

Future<BrowseApp> _pump(
  WidgetTester tester,
  FixtureLibraryRepository library, {
  Size size = phoneSize,
}) => atMockNow(() => pumpBrowseApp(tester, size, library: library));

Future<void> _trash(WidgetTester tester, String title) async {
  await tester.drag(find.text(title), const Offset(-300, 0));
  await tester.pumpAndSettle();
}

void main() {
  group('the Undo notice', () {
    testWidgets('shows after a swipe, with the words the library gave', (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Moved 1 note to the trash.');
      await _pump(tester, library);
      expect(find.byType(KansoUndoNotice), findsNothing);
      await _trash(tester, 'Sourdough starter');
      expect(find.text('Moved 1 note to the trash.'), findsOneWidget);
      expect(find.text('Undo'), findsOneWidget);
    });

    testWidgets('Undo takes the change back, and the notice goes', (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Moved 1 note to the trash.');
      final app = await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      final listed = library.listQueries.length;
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(library.undone, <int>[4]);
      expect(find.byType(KansoUndoNotice), findsNothing);
      // A bump for Undo (the list's own haptics are another object).
      expect(app.blocs.haptics.impacts, 1);
      // The library writing the change back is what makes the list read
      // again (see `Library.changes`).
      library.changeStream.add(<String>[MockIds.sourdough]);
      await tester.pumpAndSettle();
      expect(library.listQueries.length, greaterThan(listed));
    });

    testWidgets('goes by itself after a few seconds', (tester) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Pinned 1 note.');
      await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      await tester.pump(undoNoticeDuration - const Duration(seconds: 1));
      expect(find.byType(KansoUndoNotice), findsOneWidget);
      await tester.pump(const Duration(seconds: 2));
      await tester.pumpAndSettle();
      expect(find.byType(KansoUndoNotice), findsNothing);
      expect(library.undone, isEmpty);
    });

    testWidgets('a second change takes the place of the first', (tester) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(1, 'Moved 1 note to the trash.');
      await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      library.changeResult = _changed(2, 'Moved 1 note to the trash. Again.');
      await _trash(tester, 'Lisbon in spring');
      expect(find.byType(KansoUndoNotice), findsOneWidget);
      expect(find.text('Moved 1 note to the trash. Again.'), findsOneWidget);
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(library.undone, <int>[2]);
    });

    testWidgets('a change that wrote nothing shows no notice', (tester) async {
      final library = FixtureLibraryRepository()
        ..changeResult = const Right<ApplicationError, Changed>(
          Changed(count: 0, message: ''),
        );
      await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      expect(find.byType(KansoUndoNotice), findsNothing);
    });

    testWidgets("a failed undo says so in the list's own line", (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Moved 1 note to the trash.')
        ..undoResult = const Left<ApplicationError, int>(
          ApplicationError.storage(message: 'Disk error.'),
        );
      await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(find.text("That didn't work. Try again."), findsOneWidget);
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.text("That didn't work. Try again."), findsNothing);
    });

    testWidgets('is a live region and Undo a button a screen reader finds', (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Moved 1 note to the trash.');
      final handle = tester.ensureSemantics();
      await _pump(tester, library);
      await _trash(tester, 'Sourdough starter');
      expect(
        tester.getSemantics(find.text('Moved 1 note to the trash.')),
        isSemantics(
          label: 'Moved 1 note to the trash.',
          isLiveRegion: true,
        ),
      );
      expect(
        tester.getSemantics(find.text('Undo')),
        isSemantics(label: 'Undo', isButton: true, hasTapAction: true),
      );
      handle.dispose();
    });

    testWidgets('shows over the list on a tablet too, no wider than a card', (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..changeResult = _changed(4, 'Moved 1 note to the trash.');
      await _pump(tester, library, size: tabletLandscapeSize);
      await _trash(tester, 'Sourdough starter');
      final notice = tester.getRect(find.byType(KansoUndoNotice));
      expect(notice.width, lessThanOrEqualTo(420));
      expect(notice.bottom, lessThanOrEqualTo(tabletLandscapeSize.height));
      await tester.tap(find.text('Undo'));
      await tester.pumpAndSettle();
      expect(library.undone, <int>[4]);
    });
  });
}
