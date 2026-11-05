// Kanso — pdf_reader_page_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/pdf_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/pdf_reader/models/data/pdf_opening.dart';
import 'package:kanso/pdf_reader/state/pdf_reader/pdf_reader_bloc.dart';
import 'package:kanso/pdf_reader/views/pages/pdf_reader_page.dart';
import 'package:kanso/pdf_reader/views/widgets/pdf_page_view.dart';

import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/kanso_test_app.dart';
import '../../fakes.dart';

/// Opens the fixture's PDF at the first page.
const PdfReaderPage _reader = PdfReaderPage(
  noteId: 'note-one',
  file: 'file:pdf-one',
  name: 'Field notes.pdf',
);

/// A repository whose first open waits for the test to answer it.
class _PendingPdfReaderRepository extends FakePdfReaderRepository {
  final Completer<Either<ApplicationError, PdfOpening>> answer =
      Completer<Either<ApplicationError, PdfOpening>>();

  @override
  Future<Either<ApplicationError, PdfOpening>> open({
    required String noteId,
    required String file,
    String? password,
  }) {
    opens++;
    files.add(file);
    passwords.add(password);
    return answer.future;
  }
}

/// Pumps the reader in a phone-sized window.
Future<void> _pumpReader(
  WidgetTester tester, {
  Brightness brightness = Brightness.light,
}) async {
  tester.view
    ..devicePixelRatio = 3
    ..physicalSize = phoneSize * 3;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(KansoTestApp(brightness: brightness, home: _reader));
  await tester.pump();
}

/// Lets the real image decoder finish; the fake clock does not drive it.
Future<void> _drawPages(WidgetTester tester) async {
  // A page shows the spinner until its real image decode finishes, and a
  // scroll can build another page that then starts one. Let each visible
  // page decode between frames before asking the tree to settle.
  for (var round = 0; round < 4; round++) {
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 60)),
    );
    await tester.pump(const Duration(milliseconds: 150));
  }
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 60)),
  );
  await tester.pumpAndSettle();
}

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

  testWidgets('shows the loading state until the document opens', (
    tester,
  ) async {
    final pending = _PendingPdfReaderRepository();
    repository = pending;
    await _pumpReader(tester);

    expect(find.text('Opening the PDF…'), findsOneWidget);

    pending.answer.complete(
      Right<ApplicationError, PdfOpening>(
        PdfOpening.opened(
          session: FakePdfSession(),
          info: fakePdfInfo,
        ),
      ),
    );
    await tester.pump();
    await _drawPages(tester);

    expect(find.text('Page 1 of 3'), findsOneWidget);
    expect(find.text('1 / 3'), findsOneWidget);
    expect(find.byType(PdfPageView), findsWidgets);
  });

  testWidgets('a failed open can be tried again', (tester) async {
    repository.openError = const ApplicationError.storage(
      message: 'The file is busy.',
    );
    await _pumpReader(tester);
    await tester.pumpAndSettle();

    expect(find.text("This PDF can't be opened"), findsOneWidget);
    expect(find.text('The file is busy.'), findsOneWidget);
    expect(repository.opens, 1);

    repository.openError = null;
    await tester.tap(find.text('Try again'));
    await tester.pump();
    await _drawPages(tester);

    expect(repository.opens, 2);
    expect(find.text('Page 1 of 3'), findsOneWidget);
  });

  testWidgets('a password is submitted and opens the document', (
    tester,
  ) async {
    repository = FakePdfReaderRepository(
      onOpen: (password) => switch (password) {
        null => const PdfOpening.declined(
          reason: PdfDeclined.needsPassword(),
        ),
        'open sesame' => PdfOpening.opened(
          session: FakePdfSession(),
          info: fakePdfInfo,
        ),
        _ => const PdfOpening.declined(
          reason: PdfDeclined.wrongPassword(),
        ),
      },
    );
    await _pumpReader(tester);
    await tester.pumpAndSettle();

    expect(find.text('This PDF is password-protected'), findsOneWidget);
    await tester.enterText(find.byType(EditableText), 'open sesame');
    await tester.tap(find.text('Open'));
    await tester.pump();
    await _drawPages(tester);

    expect(repository.passwords, <String?>[null, 'open sesame']);
    expect(find.text('Page 1 of 3'), findsOneWidget);
  });

  testWidgets('the page chip opens the jump sheet and validates the page', (
    tester,
  ) async {
    await _pumpReader(tester);
    await tester.pump();
    await _drawPages(tester);

    await tester.tap(find.text('1 / 3'));
    await tester.pumpAndSettle();
    expect(find.text('Go to page'), findsOneWidget);

    await tester.enterText(find.byType(EditableText), '9');
    await tester.tap(find.text('Go'));
    await tester.pumpAndSettle();
    expect(find.text('Enter a page from 1 to 3.'), findsOneWidget);

    await tester.enterText(find.byType(EditableText), '3');
    await tester.tap(find.text('Go'));
    await tester.pump();
    await _drawPages(tester);

    expect(find.text('Page 3 of 3'), findsOneWidget);
  });

  testWidgets('the zoom sheet redraws the page wider than the phone', (
    tester,
  ) async {
    await _pumpReader(tester);
    await tester.pump();
    await _drawPages(tester);

    final before = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(before.width, closeTo(phoneSize.width, 0.1));

    await tester.tap(find.bySemanticsLabel('Zoom'));
    await tester.pumpAndSettle();
    expect(find.text('Fit width'), findsOneWidget);

    await tester.tap(find.text('150%'));
    await tester.pump();
    await _drawPages(tester);

    final after = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(after.width, closeTo(phoneSize.width * 1.5, 0.1));
  });

  testWidgets('dark pages invert and can be shown as printed', (tester) async {
    await _pumpReader(tester, brightness: Brightness.dark);
    await tester.pump();
    await _drawPages(tester);

    var page = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(page.invert, isTrue);
    expect(find.bySemanticsLabel('Show the page as printed'), findsOneWidget);

    await tester.tap(find.bySemanticsLabel('Show the page as printed'));
    await tester.pump();
    await _drawPages(tester);

    page = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(page.invert, isFalse);
    expect(find.bySemanticsLabel('Invert for dark mode'), findsOneWidget);
    expect(preferences.current.pdfOriginalPages['file:pdf-one'], isTrue);

    await tester.pumpWidget(
      const KansoTestApp(home: _reader),
    );
    await tester.pump(const Duration(milliseconds: 300));
    await _drawPages(tester);
    page = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(page.invert, isFalse);
    expect(find.bySemanticsLabel('Show the page as printed'), findsNothing);

    await tester.pumpWidget(
      const KansoTestApp(brightness: Brightness.dark, home: _reader),
    );
    await tester.pump(const Duration(milliseconds: 300));
    await _drawPages(tester);
    page = tester.widget<PdfPageView>(find.byType(PdfPageView).first);
    expect(page.invert, isFalse);
    expect(find.bySemanticsLabel('Invert for dark mode'), findsOneWidget);
    expect(preferences.current.pdfOriginalPages['file:pdf-one'], isTrue);
    expect(preferences.updates, 1);
  });

  testWidgets('the dark reader fits a 320 point phone', (tester) async {
    tester.view
      ..devicePixelRatio = 2
      ..physicalSize = const Size(320, 640) * 2;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(
      const KansoTestApp(brightness: Brightness.dark, home: _reader),
    );
    await tester.pump();
    await _drawPages(tester);

    expect(find.bySemanticsLabel('Search'), findsOneWidget);
    expect(find.bySemanticsLabel('Zoom'), findsOneWidget);
    expect(find.bySemanticsLabel('Show the page as printed'), findsOneWidget);
    expect(find.bySemanticsLabel('Highlights'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
