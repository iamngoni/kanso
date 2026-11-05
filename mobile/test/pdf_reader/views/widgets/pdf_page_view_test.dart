// Kanso — pdf_page_view_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/pdf_types.dart';
import 'package:kanso/pdf_reader/views/widgets/pdf_page_view.dart';

import '../../../helpers/kanso_test_app.dart';
import '../../fakes.dart';

const PdfPageSize _letter = PdfPageSize(width: 612, height: 792);

Widget _page({
  required FakePdfSession session,
  PdfSelection? selection,
  void Function(int, double, double)? onWord,
  void Function(int, double, double)? onLine,
  void Function(int, double, double, {required bool fromStart})? onExtend,
  VoidCallback? onClear,
  bool inkDrawing = false,
  void Function(int, PdfInkStroke)? onInkStroke,
}) => KansoTestApp(
  home: Scaffold(
    body: Center(
      child: SizedBox(
        width: 300,
        height: PdfPageView.heightFor(_letter, 300),
        child: PdfPageView(
          session: session,
          page: 1,
          size: _letter,
          width: 300,
          invert: false,
          marks: const <PdfMark>[
            PdfMark(
              id: 'mark-1',
              color: 'yellow',
              page: 1,
              marks: <PdfPageMarks>[
                PdfPageMarks(
                  page: 1,
                  rects: <PdfRect>[
                    PdfRect(x0: 0.1, y0: 0.2, x1: 0.8, y1: 0.23),
                  ],
                ),
              ],
            ),
          ],
          selection: selection,
          ink: const <PdfInkStroke>[],
          search: null,
          focus: null,
          onWordSelected: onWord ?? (_, _, _) {},
          onLineSelected: onLine ?? (_, _, _) {},
          onSelectionExtended: onExtend ?? (_, _, _, {required fromStart}) {},
          onSelectionCleared: onClear ?? () {},
          inkDrawing: inkDrawing,
          inkColor: '#1e1e1e',
          inkWidth: 4,
          onInkStroke: onInkStroke ?? (_, _) {},
        ),
      ),
    ),
  ),
);

/// Lets the page's pixels be decoded (the decoder is the engine's, not the
/// test clock's).
Future<void> _settlePicture(WidgetTester tester) async {
  await tester.runAsync(
    () => Future<void>.delayed(const Duration(milliseconds: 80)),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('draws the page the reader made', (tester) async {
    final session = FakePdfSession();
    await tester.pumpWidget(_page(session: session));
    await _settlePicture(tester);

    expect(session.rendered, isNotEmpty);
    expect(find.byType(RawImage), findsOneWidget);
  });

  testWidgets('holding a point selects the word there', (tester) async {
    (int, double, double)? at;
    await tester.pumpWidget(
      _page(
        session: FakePdfSession(),
        onWord: (page, x, y) => at = (page, x, y),
      ),
    );
    await _settlePicture(tester);

    await tester.longPressAt(tester.getCenter(find.byType(PdfPageView)));

    expect(at, isNotNull);
    expect(at!.$1, 1);
    expect(at!.$2, closeTo(0.5, 0.02));
    expect(at!.$3, closeTo(0.5, 0.02));
  });

  testWidgets('drawing records touch points in page units', (tester) async {
    PdfInkStroke? stroke;
    await tester.pumpWidget(
      _page(
        session: FakePdfSession(),
        inkDrawing: true,
        onInkStroke: (_, value) => stroke = value,
      ),
    );
    await _settlePicture(tester);

    final page = tester.getRect(find.byType(PdfPageView));
    final gesture = await tester.startGesture(
      Offset(page.left + page.width * 0.25, page.top + page.height * 0.5),
    );
    await gesture.moveTo(
      Offset(page.left + page.width * 0.5, page.top + page.height * 0.75),
    );
    await gesture.up();
    await tester.pump();

    expect(stroke, isNotNull);
    expect(stroke!.tool, 'pen');
    expect(stroke!.points, hasLength(2));
    expect(stroke!.points.first.x, closeTo(250, 1));
    expect(stroke!.points.first.y, closeTo(500, 1));
    expect(stroke!.points.first.pressure, 1);
    expect(stroke!.points.last.x, closeTo(500, 1));
    expect(stroke!.points.last.y, closeTo(750, 1));
  });

  testWidgets('a selection has a handle at each end', (tester) async {
    await tester.pumpWidget(
      _page(session: FakePdfSession(), selection: fakeSelection),
    );
    await _settlePicture(tester);

    expect(find.byKey(const ValueKey<String>('pdf-handle-start-1')), findsOne);
    expect(find.byKey(const ValueKey<String>('pdf-handle-end-1')), findsOne);
  });

  testWidgets('dragging the start handle extends from the other end', (
    tester,
  ) async {
    (bool, double, double)? dragged;
    await tester.pumpWidget(
      _page(
        session: FakePdfSession(),
        selection: fakeSelection,
        onExtend: (page, x, y, {required fromStart}) =>
            dragged = (fromStart, x, y),
      ),
    );
    await _settlePicture(tester);

    await tester.drag(
      find.byKey(const ValueKey<String>('pdf-handle-start-1')),
      const Offset(30, 20),
    );
    // The double-tap recognizer waits out its timeout after a press.
    await tester.pump(const Duration(milliseconds: 400));

    expect(dragged, isNotNull);
    expect(dragged!.$1, isTrue);
    expect(dragged!.$2, greaterThan(0.1));
    expect(dragged!.$3, greaterThan(0.2));
  });

  testWidgets('a tap outside a word asks for the selection to go', (
    tester,
  ) async {
    var cleared = false;
    await tester.pumpWidget(
      _page(
        session: FakePdfSession(),
        selection: fakeSelection,
        onClear: () => cleared = true,
      ),
    );
    await _settlePicture(tester);

    await tester.tap(find.byType(PdfPageView));
    // A single tap waits for a second one before it is a tap.
    await tester.pump(const Duration(milliseconds: 400));
    expect(cleared, isTrue);
  });
}
