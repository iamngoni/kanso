// Kanso — attachment_sheet_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as glados;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_page.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/views/widgets/attachment_sheet.dart';
import 'package:kanso/pdf_reader/state/pdf_reader/pdf_reader_bloc.dart';
import 'package:kanso/pdf_reader/views/pages/pdf_reader_page.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/pump.dart';
import '../../../pdf_reader/fakes.dart';

final AttachmentFile _pdf = AttachmentFile(
  id: 'file:plan',
  name: 'Garden plan 2026.pdf',
  contentType: 'application/pdf',
  bytes: Uint8List(248300),
);

void main() {
  group('byteSize', () {
    test('reads as a person reads a size', () {
      expect(byteSize(0), '0 B');
      expect(byteSize(999), '999 B');
      expect(byteSize(1000), '1.0 KB');
      expect(byteSize(3400), '3.4 KB');
      expect(byteSize(248300), '248 KB');
      expect(byteSize(12000000), '12 MB');
      expect(byteSize(5000000000000), '5000 GB');
    });

    // Property: a larger file never reads as a smaller unit.
    glados.Glados2<int, int>(
      glados.any.intInRange(0, 1 << 40),
      glados.any.intInRange(0, 1 << 40),
    ).test('the unit never goes down as the size goes up', (a, b) {
      const units = <String>['B', 'KB', 'MB', 'GB'];
      final (small, large) = a <= b ? (a, b) : (b, a);
      int unitOf(int n) => units.indexOf(byteSize(n).split(' ').last);
      expect(unitOf(small), lessThanOrEqualTo(unitOf(large)));
    });
  });

  for (final brightness in Brightness.values) {
    testWidgets("an attachment's sheet, ${brightness.name}", (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        Scaffold(
          body: Align(
            alignment: Alignment.bottomCenter,
            child: Material(child: AttachmentSheet(file: _pdf)),
          ),
        ),
        name: 'phone_attachment_sheet_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }

  testWidgets('image attachment is full-screen, black and system-backable', (
    tester,
  ) async {
    final image = AttachmentFile(
      id: 'file:image',
      name: 'photo.png',
      contentType: 'image/png',
      bytes: Uint8List.fromList(<int>[
        137,
        80,
        78,
        71,
        13,
        10,
        26,
        10,
        0,
        0,
        0,
        13,
        73,
        72,
        68,
        82,
        0,
        0,
        0,
        1,
        0,
        0,
        0,
        1,
        8,
        6,
        0,
        0,
        0,
        31,
        21,
        196,
        137,
        0,
        0,
        0,
        13,
        73,
        68,
        65,
        84,
        8,
        215,
        99,
        248,
        207,
        192,
        240,
        31,
        0,
        5,
        0,
        1,
        255,
        137,
        153,
        61,
        29,
        0,
        0,
        0,
        0,
        73,
        69,
        78,
        68,
        174,
        66,
        96,
        130,
      ]),
    );
    await pumpKanso(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => showAttachment(context, image),
          child: const Text('Open image'),
        ),
      ),
    );

    await tester.tap(find.text('Open image'));
    await tester.pumpAndSettle();

    expect(find.byType(AttachmentImagePage), findsOneWidget);
    expect(find.byType(KansoPage), findsOneWidget);
    expect(find.byType(InteractiveViewer), findsOneWidget);
    expect(find.text('Done'), findsOneWidget);
    expect(find.text('Share'), findsOneWidget);
    final shell = tester.widget<KansoPage>(find.byType(KansoPage));
    expect(shell.backgroundColor, Colors.black);
    expect(shell.resizeToAvoidBottomInset, isFalse);

    await tester.binding.handlePopRoute();
    await tester.pumpAndSettle();
    expect(find.byType(AttachmentImagePage), findsNothing);
  });

  testWidgets('a PDF with its note opens in the reader', (tester) async {
    getIt.registerFactory<PdfReaderBloc>(
      () => PdfReaderBloc(
        FakePdfReaderRepository(),
        FakePdfPreferencesRepository(),
      ),
    );
    addTearDown(() async => getIt.reset());
    await pumpKanso(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => showAttachment(context, _pdf, noteId: 'note-one'),
          child: const Text('Open PDF'),
        ),
      ),
    );

    await tester.tap(find.text('Open PDF'));
    await tester.pump();

    expect(find.byType(PdfReaderPage), findsOneWidget);
  });

  testWidgets('a PDF without its note keeps the attachment sheet', (
    tester,
  ) async {
    await pumpKanso(
      tester,
      Builder(
        builder: (context) => TextButton(
          onPressed: () => showAttachment(context, _pdf),
          child: const Text('Open PDF'),
        ),
      ),
    );

    await tester.tap(find.text('Open PDF'));
    await tester.pumpAndSettle();

    expect(find.byType(AttachmentSheet), findsOneWidget);
    expect(find.text('Garden plan 2026.pdf'), findsOneWidget);
  });
}
