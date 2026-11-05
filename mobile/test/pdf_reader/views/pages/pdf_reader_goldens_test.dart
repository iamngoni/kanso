// Kanso — pdf_reader_goldens_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/pdf_types.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/pdf_reader/models/data/pdf_opening.dart';
import 'package:kanso/pdf_reader/state/pdf_reader/pdf_reader_bloc.dart';
import 'package:kanso/pdf_reader/views/pages/pdf_reader_page.dart';
import 'package:kanso/pdf_reader/views/widgets/pdf_page_view.dart';

import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/golden.dart';
import '../../fakes.dart';

/// The page a reader test opens: the fixture's PDF on its first page.
Widget _reader() => const PdfReaderPage(
  noteId: 'note-one',
  file: 'file:pdf-one',
  name: 'Field notes.pdf',
);

/// Waits for the real image decoder, which the fake clock does not drive.
Future<void> _drawPages(WidgetTester tester) async {
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 100)),
  );
  await tester.pumpAndSettle();
}

/// The sizes the reader is laid out at: a phone and the two iPad
/// orientations the app supports.
const Map<String, Size> _sizes = <String, Size>{
  'phone': phoneSize,
  'ipad_portrait': tabletPortraitSize,
  'ipad_landscape': tabletLandscapeSize,
};

void main() {
  late FakePdfReaderRepository repository;
  late FakePdfPreferencesRepository preferences;

  setUp(() {
    repository = FakePdfReaderRepository();
    preferences = FakePdfPreferencesRepository();
    final locks = FixtureSessionRepository(unlocked: true);
    getIt
      ..registerSingleton<SessionRepository>(locks)
      ..registerFactory<PdfReaderBloc>(
        () => PdfReaderBloc(repository, preferences, locks),
      );
    addTearDown(() async {
      await locks.close();
      await getIt.reset();
    });
  });

  for (final brightness in Brightness.values) {
    for (final entry in _sizes.entries) {
      testWidgets(
        'the reader, ${entry.key}, ${brightness.name}',
        (tester) async {
          await expectGoldenOnBothPlatforms(
            tester,
            _reader(),
            name: '${entry.key}_pdf_reader_${brightness.name}',
            size: entry.value,
            brightness: brightness,
            interact: _drawPages,
          );

          expect(find.text('Field notes.pdf'), findsOneWidget);
          expect(find.byType(PdfPageView), findsWidgets);
          expect(find.text('1 / 3'), findsOneWidget);
        },
      );
    }

    testWidgets('a selection, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _reader(),
        name: 'phone_pdf_reader_selection_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
        interact: (tester) async {
          await _drawPages(tester);
          await tester.longPressAt(
            tester.getCenter(find.byType(PdfPageView).first),
          );
          await tester.pumpAndSettle();
        },
      );

      expect(find.text('the words of a page'), findsOneWidget);
      expect(find.text('Add quote to note'), findsOneWidget);
      expect(find.text('Highlight'), findsOneWidget);
    });

    testWidgets('the password panel, ${brightness.name}', (tester) async {
      repository.opening = const PdfOpening.declined(
        reason: PdfDeclined.needsPassword(),
      );
      await expectGoldenOnBothPlatforms(
        tester,
        _reader(),
        name: 'phone_pdf_reader_password_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );

      expect(find.text('This PDF is password-protected'), findsOneWidget);
      expect(find.text('Open'), findsOneWidget);
    });

    testWidgets('the locked-note message, ${brightness.name}', (tester) async {
      repository.opening = const PdfOpening.declined(
        reason: PdfDeclined.lockedNote(
          message: 'Unlock this note to open its PDF.',
        ),
      );
      await expectGoldenOnBothPlatforms(
        tester,
        _reader(),
        name: 'phone_pdf_reader_locked_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );

      expect(find.text('Locked'), findsOneWidget);
      expect(find.text('Unlock this note to open its PDF.'), findsOneWidget);
    });

    testWidgets('the search result, ${brightness.name}', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        _reader(),
        name: 'phone_pdf_reader_search_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
        interact: (tester) async {
          await _drawPages(tester);
          await tester.tap(find.bySemanticsLabel('Search'));
          await tester.pumpAndSettle();
          await tester.enterText(find.byType(EditableText), 'page');
          await tester.pump(const Duration(milliseconds: 300));
          await tester.pumpAndSettle();
        },
      );

      expect(find.text('1 of 2'), findsOneWidget);
      expect(find.text('1 page has no text'), findsOneWidget);
    });
  }
}
