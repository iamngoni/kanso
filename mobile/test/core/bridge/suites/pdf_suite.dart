// Kanso — pdf_suite.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The phone's PDF reader, through the bridge: opening a real attachment,
/// drawing and searching its text, selecting by touch and saving a highlight.
void pdfSuite(OpenLibrary open) {
  group('KansoPdf', () {
    test('an open reader stops returning data when its note locks', () async {
      final library = await open();
      final id = await makeNote(library, 'Private report', 'Read it.');
      final editor = await library.edit(id: id);
      await editor.attach(
        file: ShareFile(
          name: 'Private-report.pdf',
          contentType: 'application/pdf',
          bytes: _reportPdf(),
        ),
      );
      await editor.close();
      final body = (await library.note(id: id)).body ?? '';
      final linked = RegExp('inkdrop://file:([A-Za-z0-9_-]+)').firstMatch(body);
      final file = 'file:${linked!.group(1)}';

      final locks = library.locks();
      await locks.lockNote(id: id, password: 'correct horse');
      expect(
        await locks.unlockNote(id: id, password: 'correct horse'),
        isTrue,
      );
      final opened = await library.pdf().open(noteId: id, file: file);
      final reader = opened.reader!;
      addTearDown(() {
        reader
          ..close()
          ..dispose();
      });
      expect((await reader.find(query: 'Quarterly', limit: 10)).total, 1);

      await locks.lock();
      await expectLater(
        reader.find(query: 'Quarterly', limit: 10),
        throwsA(isA<KansoError_Locked>()),
      );
      await expectLater(
        reader.render(page: 0, widthPx: 612, invert: false),
        throwsA(isA<KansoError_Locked>()),
      );
    });

    test(
      'opens a PDF attachment and reads, searches and highlights it',
      () async {
        final library = await open();
        final id = await makeNote(
          library,
          'Quarterly report',
          'Read the report.',
        );
        final editor = await library.edit(id: id);
        final attached = await editor.attach(
          file: ShareFile(
            name: 'Quarterly-report.pdf',
            contentType: 'application/pdf',
            bytes: _reportPdf(),
          ),
        );
        expect(attached.replace, isNotNull);
        await editor.close();

        final body = (await library.note(id: id)).body ?? '';
        final linked = RegExp(
          'inkdrop://file:([A-Za-z0-9_-]+)',
        ).firstMatch(body);
        expect(linked, isNotNull);
        final file = 'file:${linked!.group(1)}';

        final pdf = library.pdf();
        final before = await pdf.marks(noteId: id, file: file);
        expect(before.editable, isTrue);
        expect(before.marks, isEmpty);
        expect(before.rev, isNotEmpty);

        final result = await pdf.open(noteId: id, file: file);
        expect(result.declined, isNull);
        final reader = result.reader!;
        final info = result.info!;
        addTearDown(() {
          reader
            ..close()
            ..dispose();
        });

        expect(info.file, file);
        expect(info.name, 'Quarterly-report.pdf');
        expect(info.pages, 1);
        expect(info.sizes.single.width, 612);
        expect(info.sizes.single.height, 792);

        final page = await reader.render(page: 0, widthPx: 612, invert: false);
        expect(page.page, 0);
        expect(page.width, 612);
        expect(page.height, 792);
        expect(page.bgra.length, page.width * page.height * 4);

        final found = await reader.find(query: 'Quarterly', limit: 10);
        expect(found.total, 1);
        expect(found.pagesWithoutText, 0);
        expect(found.hits.single.page, 1);
        final rect = found.hits.single.marks.single.rects.single;
        final x = (rect.x0 + rect.x1) / 2;
        final y = (rect.y0 + rect.y1) / 2;

        final word = await reader.wordAt(page: 1, x: x, y: y);
        expect(word, isNotNull);
        expect(word!.text, 'Quarterly');
        expect(word.start.page, 1);

        final line = await reader.lineAt(page: 1, x: x, y: y);
        expect(line, isNotNull);
        expect(line!.text, 'Quarterly report');

        final selected = await reader.select(
          start: word.start,
          end: word.end,
        );
        expect(selected, isNotNull);
        expect(selected!.text, word.text);

        final report = await reader.find(query: 'report', limit: 10);
        expect(report.total, 1);
        final reportRect = report.hits.single.marks.single.rects.single;
        final extended = await reader.extend(
          anchor: word.start,
          page: 1,
          x: (reportRect.x0 + reportRect.x1) / 2,
          y: (reportRect.y0 + reportRect.y1) / 2,
        );
        expect(extended, isNotNull);
        expect(extended!.text, startsWith('Quarterly'));
        expect(extended.text.length, greaterThan(word.text.length));

        final made = await reader.addHighlight(
          baseRev: before.rev,
          start: word.start,
          end: word.end,
          color: 'yellow',
          quote: false,
        );
        expect(made.id, isNotNull);
        expect(made.gone, isFalse);
        expect(made.merged, isFalse);

        final after = await pdf.marks(noteId: id, file: file);
        expect(after.marks, hasLength(1));
        expect(after.marks.single.color, 'yellow');
        expect(after.marks.single.marks, isNotEmpty);
      },
    );
  });
}

/// A one-page Letter PDF with a line of Helvetica text. Built here rather
/// than kept as a binary fixture, with the xref offsets the format requires.
Uint8List _reportPdf() {
  const content = 'BT /F1 24 Tf 72 700 Td (Quarterly report) Tj ET';
  final objects = <String>[
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [4 0 R] /Count 1 >>',
    '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents 5 0 R >>',
    '<< /Length ${content.length} >>\nstream\n$content\nendstream',
  ];
  final out = BytesBuilder()..add(utf8.encode('%PDF-1.4\n'));
  final offsets = <int>[];
  for (var i = 0; i < objects.length; i++) {
    offsets.add(out.length);
    out.add(utf8.encode('${i + 1} 0 obj\n${objects[i]}\nendobj\n'));
  }
  final xref = out.length;
  out.add(
    utf8.encode(
      'xref\n0 ${objects.length + 1}\n'
      '0000000000 65535 f \n',
    ),
  );
  for (final offset in offsets) {
    out.add(utf8.encode('${offset.toString().padLeft(10, '0')} 00000 n \n'));
  }
  out.add(
    utf8.encode(
      'trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\n'
      'startxref\n$xref\n%%EOF\n',
    ),
  );
  return out.takeBytes();
}
