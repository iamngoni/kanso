// Kanso — pdf_highlights_sheet_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' hide State;
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as glados;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/note/views/widgets/pdf_highlights_sheet.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/property.dart';
import '../../../helpers/pump.dart';

const PdfNotes _report = PdfNotes(
  file: 'file:abc123',
  name: 'Quarterly report.pdf',
  url: 'inkdrop://file:abc123',
  editable: true,
  highlights: <PdfHighlightRow>[
    PdfHighlightRow(
      id: 'h7k2m9ab',
      page: 4,
      color: 'yellow',
      text: 'Revenue rose 12 percent on the quarter, led by subscriptions.',
      comment: 'Check against the board deck.',
      inNote: true,
    ),
    PdfHighlightRow(
      id: 'k3p8q1zz',
      page: 5,
      color: 'pink',
      text: 'Costs held flat while the team grew by three people.',
      inNote: false,
    ),
    PdfHighlightRow(
      id: 'm5n2b7cc',
      page: 9,
      color: 'blue',
      text: 'The pilot in the north region finished early.',
      inNote: true,
    ),
  ],
  ink: <PdfInkRow>[
    PdfInkRow(page: 4, strokes: 12),
    PdfInkRow(page: 7, strokes: 1),
  ],
);

FixtureLibraryRepository _library(List<PdfNotes> notes) {
  final library = FixtureLibraryRepository()
    ..pdfHighlightsResult = Right<ApplicationError, List<PdfNotes>>(notes);
  return library;
}

Widget _sheet(FixtureLibraryRepository library) => Scaffold(
  body: Align(
    alignment: Alignment.bottomCenter,
    child: Builder(
      builder: (context) => ColoredBox(
        color: context.colors.elevated,
        child: PdfHighlightsSheet(
          noteId: library.notes.keys.first,
          repository: library,
        ),
      ),
    ),
  ),
);

void main() {
  glados.Glados<String>(
    glados.any.lowercaseLetters,
    kansoExploreConfig(),
  ).test(
    'a highlight colour is always one of the four, yellow when unknown',
    (name) {
      const known = <String, int>{
        'yellow': 0xFFFFD60A,
        'green': 0xFF34C759,
        'blue': 0xFF3B9CFF,
        'pink': 0xFFFF5C9A,
      };
      expect(highlightColor(name).toARGB32(), known[name] ?? 0xFFFFD60A);
    },
  );

  for (final brightness in Brightness.values) {
    testWidgets('the highlights sheet, ${brightness.name}', (tester) async {
      final library = _library(<PdfNotes>[_report]);
      await expectGoldenOnBothPlatforms(
        tester,
        _sheet(library),
        name: 'phone_pdf_highlights_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });

    testWidgets('a highlight opened, ${brightness.name}', (tester) async {
      final library = _library(<PdfNotes>[_report]);
      await expectGoldenOnBothPlatforms(
        tester,
        _sheet(library),
        name: 'phone_pdf_highlight_open_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
        interact: (tester) async {
          await tester.tap(
            find.byKey(const ValueKey<String>('pdf-row-k3p8q1zz')),
          );
          await tester.pumpAndSettle();
        },
      );
    });
  }

  testWidgets('rows show the page, the comment and what is not in the note', (
    tester,
  ) async {
    final library = _library(<PdfNotes>[_report]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(find.text('Quarterly report.pdf'), findsOneWidget);
    expect(find.text('Page 4'), findsOneWidget);
    expect(find.text('Check against the board deck.'), findsOneWidget);
    expect(
      find.byKey(const ValueKey<String>('pdf-not-in-note-k3p8q1zz')),
      findsOneWidget,
    );
    expect(
      find.byKey(const ValueKey<String>('pdf-not-in-note-h7k2m9ab')),
      findsNothing,
    );
  });

  testWidgets(
    'a comment, a colour, the quote and a delete are one write each',
    (
      tester,
    ) async {
      tester.view.physicalSize = const Size(800, 3000);
      addTearDown(tester.view.reset);
      final library = _library(<PdfNotes>[_report]);
      await pumpKanso(tester, _sheet(library));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey<String>('pdf-row-k3p8q1zz')));
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(EditableText), 'Ask finance');
      await tester.tap(find.text('Save note'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const ValueKey<String>('pdf-color-green')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Add quote to the note'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Delete highlight'));
      await tester.pumpAndSettle();

      expect(
        library.pdfChanges.map((c) => (c.$3, c.$4)).toSet(),
        <(String, String)>{
          ('file:abc123', 'k3p8q1zz'),
        },
      );
      expect(library.pdfChanges.map((c) => c.$5).toList(), <PdfHighlightChange>[
        const PdfHighlightChange.comment(comment: 'Ask finance'),
        const PdfHighlightChange.color(color: 'green'),
        const PdfHighlightChange.addQuote(),
        const PdfHighlightChange.delete(),
      ]);
      // Each is made on the revision the note has when it is made.
      expect(
        library.pdfChanges.map((c) => c.$2).every((rev) => rev.isNotEmpty),
        isTrue,
      );
    },
  );

  testWidgets('highlights that cannot be read are shown without edits', (
    tester,
  ) async {
    final library = _library(<PdfNotes>[
      const PdfNotes(
        file: 'file:abc123',
        name: 'Quarterly report.pdf',
        url: 'inkdrop://file:abc123',
        editable: false,
        why: 'broken',
        highlights: <PdfHighlightRow>[
          PdfHighlightRow(
            id: 'h7k2m9ab',
            page: 4,
            color: 'yellow',
            text: 'Revenue rose.',
            inNote: true,
          ),
        ],
        ink: <PdfInkRow>[PdfInkRow(page: 4, strokes: 3)],
      ),
    ]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(
      find.byKey(const ValueKey<String>('pdf-notes-read-only')),
      findsOneWidget,
    );
    await tester.tap(find.byKey(const ValueKey<String>('pdf-row-h7k2m9ab')));
    await tester.pumpAndSettle();
    expect(find.text('Delete highlight'), findsNothing);
  });

  testWidgets('pencil pages are listed and one can be cleared', (tester) async {
    tester.view.physicalSize = const Size(800, 3000);
    addTearDown(tester.view.reset);
    final library = _library(<PdfNotes>[_report]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(find.text('Pencil marks'), findsOneWidget);
    expect(find.text('Page 4 · 12 strokes'), findsOneWidget);
    expect(find.text('Page 7 · 1 stroke'), findsOneWidget);

    await tester.tap(
      find.byKey(const ValueKey<String>('pdf-ink-clear-file:abc123-7')),
    );
    await tester.pumpAndSettle();
    expect(library.pdfInkCleared.map((c) => (c.$3, c.$4)).toList(), <
      (
        String,
        int,
      )
    >[
      ('file:abc123', 7),
    ]);
    expect(library.pdfInkCleared.single.$2, isNotEmpty);
  });

  testWidgets('pencil marks that cannot be edited have no clear button', (
    tester,
  ) async {
    final library = _library(<PdfNotes>[
      const PdfNotes(
        file: 'file:abc123',
        name: 'Quarterly report.pdf',
        url: 'inkdrop://file:abc123',
        editable: false,
        why: 'broken',
        highlights: <PdfHighlightRow>[],
        ink: <PdfInkRow>[PdfInkRow(page: 2, strokes: 5)],
      ),
    ]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(find.text('Page 2 · 5 strokes'), findsOneWidget);
    expect(
      find.byKey(const ValueKey<String>('pdf-ink-clear-file:abc123-2')),
      findsNothing,
    );
  });

  testWidgets('a note with only pencil marks does not say it is empty', (
    tester,
  ) async {
    final library = _library(<PdfNotes>[
      const PdfNotes(
        file: 'file:abc123',
        name: 'Quarterly report.pdf',
        url: 'inkdrop://file:abc123',
        editable: true,
        highlights: <PdfHighlightRow>[],
        ink: <PdfInkRow>[PdfInkRow(page: 1, strokes: 2)],
      ),
    ]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(find.text('This note has no highlights.'), findsNothing);
    expect(find.text('Page 1 · 2 strokes'), findsOneWidget);
  });

  testWidgets('a note with no highlights says so', (tester) async {
    final library = _library(const <PdfNotes>[]);
    await pumpKanso(tester, _sheet(library));
    await tester.pumpAndSettle();
    expect(find.text('This note has no highlights.'), findsOneWidget);
  });
}
